use std::{process::Command, sync::mpsc::Sender};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
pub use miette::Result;
use miette::{miette, Context, IntoDiagnostic};
use openai::{
    chat::{complete_chat, ChatMessage, ChatRole},
    OpenAiConfig,
};
use tracing_common::setup_tracing;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};

const PREFERRED_MIC_NAME: &str = "Samson G-Track Pro";

fn recording_thread_main(string_sender: Sender<String>) -> Result<()> {
    loop {
        let host = cpal::default_host();

        let device = host
            .input_devices()
            .into_diagnostic()?
            .find(|device| match device.name() {
                Ok(name) => name == PREFERRED_MIC_NAME,
                Err(err) => {
                    eprintln!("Failed to get device name: {}", err);
                    false
                }
            })
            .ok_or_else(|| {
                miette::miette!("No input device found with name {}", PREFERRED_MIC_NAME)
            })?;

        println!("Input device: {}", device.name().into_diagnostic()?);

        let config = device
            .default_input_config()
            .into_diagnostic()
            .wrap_err("Failed to get default input config")?;
        println!("Default input config: {:?}", config);

        println!("Begin recording...");

        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };

        let (sender, receiver) = std::sync::mpsc::channel::<Vec<f32>>();

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device
                .build_input_stream(
                    &config.into(),
                    move |data, _: &_| sender.send(data.to_vec()).unwrap(),
                    err_fn,
                    None,
                )
                .into_diagnostic()?,
            sample_format => {
                return Err(miette::miette!(
                    "Unsupported sample format '{sample_format}'"
                ))
            }
        };

        let path_to_model = concat!(env!("CARGO_MANIFEST_DIR"), "/../models/ggml-base.en.bin");

        // load a context and model
        let ctx = WhisperContext::new(path_to_model).expect("failed to load model");

        stream.play().into_diagnostic()?;

        let mut recorded_sample = vec![];
        while let Ok(mut data) = receiver.recv() {
            recorded_sample.append(&mut data);

            if recorded_sample.len() >= 480000 {
                let resampled = samplerate::convert(
                    48000,
                    16000,
                    2,
                    samplerate::ConverterType::SincBestQuality,
                    &recorded_sample,
                )
                .unwrap();
                let pcm_data = convert_stereo_to_mono_audio(&resampled).map_err(|e| miette!(e))?;
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
                    .full(params, &pcm_data[..])
                    .expect("failed to run model");

                let num_segments = state
                    .full_n_segments()
                    .expect("failed to get number of segments");
                for i in 0..num_segments {
                    if let Ok(segment) = state.full_get_segment_text(i) {
                        string_sender.send(segment).unwrap();
                    }
                }

                recorded_sample.clear();
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing()?;

    let (sender, reciever) = std::sync::mpsc::channel::<String>();

    let _recording = tokio::task::spawn_blocking(move || {
        let _ = recording_thread_main(sender);
    });

    let (message_sender, message_reciever) = std::sync::mpsc::channel::<String>();

    let _detecting = tokio::task::spawn_blocking(move || {
        let mut buffer = vec![];
        let mut buffering = false;
        loop {
            let text = reciever.recv().unwrap();
            let text = text.to_lowercase();
            // println!("{}", text);

            if buffering {
                buffer.push(text);
                buffering = false;

                message_sender.send(buffer.join(" ")).unwrap();
                buffer.clear();
            } else if text.contains("bite") || text.contains("byte") {
                println!("Bite detected!");
                buffer.push(text);
                buffering = true
            }
        }
    });

    let openai_config = OpenAiConfig::from_env()?;
    loop {
        while let Ok(message) = message_reciever.try_recv() {
            println!("{}", message);

            let messages = vec![
                ChatMessage {
                    role: ChatRole::System,
                    content: r#"
                The following message was recorded and transcribed during a live chat.
                I have a bot named Byte (or maybe Bite) that I might be trying to talk to.
                Remeber this is a AI transcribed audio, so there may be errors in the transcription.
                
                You are a helpful chatbot and should respond as Byte.
                You are my stream companion and can answer any question I ask
                Do NOT comment on the spelling of your name under any circumstances

                If I didn't ask a question keep your answer short and consise
                "#
                    .to_string(),
                },
                ChatMessage {
                    role: ChatRole::User,
                    content: message,
                },
            ];
            let resp = complete_chat(&openai_config, "gpt-3.5-turbo", messages).await?;
            // println!("Response: {}", resp.content);

            Command::new("say")
                .arg(resp.content)
                .spawn()
                .expect("failed to run say");
        }
    }

    Ok(())
}

pub fn convert_stereo_to_mono_audio(samples: &[f32]) -> Result<Vec<f32>, &'static str> {
    if samples.len() & 1 != 0 {
        return Err("The stereo audio vector has an odd number of samples. \
            This means a half-sample is missing somewhere");
    }

    Ok(samples
        .chunks_exact(2)
        .map(|x| (x[0] + x[1]) / 2.0)
        .collect())
}
