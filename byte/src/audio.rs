use color_eyre::eyre::Context;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use openai::chat::{complete_chat, ChatMessage, ChatRole};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};

use crate::Config;

const PREFERRED_MIC_NAME: &str = "Samson G-Track Pro";

fn recording_thread_main(
    string_sender: &tokio::sync::mpsc::Sender<String>,
) -> color_eyre::Result<()> {
    let host = cpal::default_host();

    let device = host
        .input_devices()?
        .find(|device| match device.name() {
            Ok(name) => name == PREFERRED_MIC_NAME,
            Err(err) => {
                eprintln!("Failed to get device name: {err}");
                false
            }
        })
        .ok_or_else(|| {
            color_eyre::eyre::eyre!("No input device found with name {}", PREFERRED_MIC_NAME)
        })?;

    println!("Input device: {}", device.name()?);

    let config = device
        .default_input_config()
        .wrap_err("Failed to get default input config")?;
    println!("Default input config: {config:?}");

    println!("Begin recording...");

    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {err}");
    };

    let (sender, receiver) = std::sync::mpsc::channel::<Vec<f32>>();

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.clone().into(),
            move |data, _: &_| sender.send(data.to_vec()).unwrap(),
            err_fn,
            None,
        )?,
        sample_format => {
            return Err(color_eyre::eyre::eyre!(
                "Unsupported sample format '{sample_format}'"
            ))
        }
    };

    let path_to_model = concat!(env!("CARGO_MANIFEST_DIR"), "/../models/ggml-base.en.bin");

    // load a context and model
    let ctx = WhisperContext::new(path_to_model).expect("failed to load model");

    stream.play()?;

    let mut recorded_sample = vec![];
    while let Ok(mut data) = receiver.recv() {
        recorded_sample.append(&mut data);

        if recorded_sample.len() >= 480_000 {
            let audio_data = &recorded_sample[..];
            let audio_data = if config.sample_rate().0 == 16_000 {
                audio_data.to_vec()
            } else {
                samplerate::convert(
                    config.sample_rate().0,
                    16000,
                    config.channels().into(),
                    samplerate::ConverterType::SincBestQuality,
                    audio_data,
                )
                .wrap_err("Failed to convert to 16kHz")?
            };
            let audio_data = match config.channels() {
                1 => audio_data,
                2 => convert_stereo_to_mono_audio(&audio_data)
                    .map_err(|e| color_eyre::eyre::eyre!(e))?,
                _ => {
                    return Err(color_eyre::eyre::eyre!(
                        "Unsupported number of channels: {}",
                        config.channels()
                    ))
                }
            };
            let mut params = FullParams::new(SamplingStrategy::BeamSearch {
                beam_size: 5,
                patience: 0.1,
            });
            params.set_print_special(false);
            params.set_print_progress(false);
            params.set_print_realtime(false);
            params.set_print_timestamps(false);

            let mut state = ctx.create_state().expect("failed to create state");
            state
                .full(params, &audio_data[..])
                .expect("failed to run model");

            let num_segments = state
                .full_n_segments()
                .expect("failed to get number of segments");
            for i in 0..num_segments {
                if let Ok(segment) = state.full_get_segment_text(i) {
                    println!("Segment {i}: {segment}");
                    string_sender.blocking_send(segment).unwrap();
                }
            }

            recorded_sample.clear();
        }
    }

    unreachable!("The audio recording channel should never close");
}

pub(crate) async fn run_audio_loop(config: Config) -> color_eyre::Result<()> {
    let (sender, mut reciever) = tokio::sync::mpsc::channel::<String>(32);

    std::thread::spawn(move || recording_thread_main(&sender));

    let (message_sender, mut message_reciever) = tokio::sync::mpsc::channel::<String>(32);

    let _detecting = tokio::task::spawn(async move {
        while let Some(text) = reciever.recv().await {
            let text = text.to_lowercase();
            println!("{text}");

            if text.contains("bite") || text.contains("byte") {
                println!("Bite detected!");
                let next_text = reciever.recv().await.unwrap();

                message_sender
                    .send(format!("{text} {next_text}"))
                    .await
                    .unwrap();
            }
        }
    });

    while let Some(message) = message_reciever.recv().await {
        println!("{message}");

        let messages = vec![
            ChatMessage {
                role: ChatRole::System,
                content: r"
                The following message was recorded and transcribed during a live chat.
                I have a bot named Byte (or maybe Bite) that I might be trying to talk to.
                Remeber this is a AI transcribed audio, so there may be errors in the transcription.
                
                You are a helpful chatbot and should respond as Byte.
                You are my stream companion and can answer any question I ask
                Do NOT comment on the spelling of your name under any circumstances

                If I didn't ask a question keep your answer short and consise
                "
                .to_string(),
            },
            ChatMessage {
                role: ChatRole::User,
                content: message,
            },
        ];
        let resp = complete_chat(&config.openai, "gpt-3.5-turbo", messages).await?;
        // println!("Response: {}", resp.content);

        config.say.send(resp.content).await?;
    }

    unreachable!("The message channel should never close");
}

fn convert_stereo_to_mono_audio(samples: &[f32]) -> Result<Vec<f32>, &'static str> {
    if samples.len() & 1 != 0 {
        return Err("The stereo audio vector has an odd number of samples. \
            This means a half-sample is missing somewhere");
    }

    Ok(samples
        .chunks_exact(2)
        .map(|x| (x[0] + x[1]) / 2.0)
        .collect())
}
