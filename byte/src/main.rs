use std::{
    fs::File,
    io::BufWriter,
    path::Path,
    sync::{Arc, Mutex},
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, Sample,
};
use hound::WavReader;
pub use miette::Result;
use miette::{miette, Context, IntoDiagnostic};
use tracing_common::setup_tracing;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};

const PREFERRED_MIC_NAME: &str = "Samson G-Track Pro";

#[tokio::main]
async fn main() -> Result<()> {
    let path_to_model = concat!(env!("CARGO_MANIFEST_DIR"), "/../models/ggml-large.bin");

    // load a context and model
    let ctx = WhisperContext::new(path_to_model).expect("failed to load model");

    setup_tracing()?;

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
        .ok_or_else(|| miette::miette!("No input device found with name {}", PREFERRED_MIC_NAME))?;

    println!("Input device: {}", device.name().into_diagnostic()?);

    let config = device
        .default_input_config()
        .into_diagnostic()
        .wrap_err("Failed to get default input config")?;
    println!("Default input config: {:?}", config);

    // The WAV file we're recording to.
    const PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tmp/recorded.wav");
    let spec = wav_spec_from_config(&config);
    let writer = hound::WavWriter::create(PATH, spec)
        .into_diagnostic()
        .wrap_err("Could not make Hound writer")?;
    let writer = Arc::new(Mutex::new(Some(writer)));

    // A flag to indicate that recording is in progress.
    println!("Begin recording...");

    // Run the input stream on a separate thread.
    let writer_2 = writer.clone();

    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };

    let stream = match config.sample_format() {
        cpal::SampleFormat::I8 => device
            .build_input_stream(
                &config.into(),
                move |data, _: &_| write_input_data::<i8, i8>(data, &writer_2),
                err_fn,
                None,
            )
            .into_diagnostic()?,
        cpal::SampleFormat::I16 => device
            .build_input_stream(
                &config.into(),
                move |data, _: &_| write_input_data::<i16, i16>(data, &writer_2),
                err_fn,
                None,
            )
            .into_diagnostic()?,
        cpal::SampleFormat::I32 => device
            .build_input_stream(
                &config.into(),
                move |data, _: &_| write_input_data::<i32, i32>(data, &writer_2),
                err_fn,
                None,
            )
            .into_diagnostic()?,
        cpal::SampleFormat::F32 => device
            .build_input_stream(
                &config.into(),
                move |data, _: &_| write_input_data::<f32, f32>(data, &writer_2),
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

    stream.play().into_diagnostic()?;

    // Let recording go for roughly three seconds.
    std::thread::sleep(std::time::Duration::from_secs(3));
    drop(stream);
    writer
        .lock()
        .map_err(|_| miette::miette!("Failed to lock writer"))?
        .take()
        .ok_or_else(|| miette::miette!("Writer is None"))?
        .finalize()
        .into_diagnostic()?;
    println!("Recording {} complete!", PATH);
    drop(writer);

    {
        let params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        let path = std::path::PathBuf::from(PATH);
        let data = parse_wav_file(&path);
        let resampled = samplerate::convert(
            48000,
            16000,
            2,
            samplerate::ConverterType::SincBestQuality,
            &data,
        )
        .unwrap();
        let pcm_data = convert_stereo_to_mono_audio(&resampled).map_err(|e| miette!(e))?;

        let mut state = ctx.create_state().expect("failed to create state");
        state
            .full(params, &pcm_data[..])
            .expect("failed to run model");

        let num_segments = state
            .full_n_segments()
            .expect("failed to get number of segments");
        for i in 0..num_segments {
            let segment = state
                .full_get_segment_text(i)
                .expect("failed to get segment");
            let start_timestamp = state
                .full_get_segment_t0(i)
                .expect("failed to get segment start timestamp");
            let end_timestamp = state
                .full_get_segment_t1(i)
                .expect("failed to get segment end timestamp");
            println!("[{} - {}]: {}", start_timestamp, end_timestamp, segment);
        }
    }

    Ok(())
}

fn wav_spec_from_config(config: &cpal::SupportedStreamConfig) -> hound::WavSpec {
    hound::WavSpec {
        channels: config.channels() as _,
        sample_rate: config.sample_rate().0 as _,
        bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        sample_format: sample_format(config.sample_format()),
    }
}

fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
    if format.is_float() {
        hound::SampleFormat::Float
    } else {
        hound::SampleFormat::Int
    }
}

type WavWriterHandle = Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>;

fn write_input_data<T, U>(input: &[T], writer: &WavWriterHandle)
where
    T: Sample,
    U: Sample + hound::Sample + FromSample<T>,
{
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in input.iter() {
                let sample: U = U::from_sample(sample);
                writer.write_sample(sample).ok();
            }
        }
    }
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

fn parse_wav_file(path: &Path) -> Vec<f32> {
    let reader = WavReader::open(path).expect("failed to read file");

    // if reader.spec().channels != 1 {
    //     panic!("expected mono audio file");
    // }
    // if reader.spec().sample_format != SampleFormat::Int {
    //     panic!("expected integer sample format");
    // }
    // if reader.spec().sample_rate != 16000 {
    //     panic!("expected 16KHz sample rate");
    // }
    // if reader.spec().bits_per_sample != 16 {
    //     panic!("expected 16 bits per sample");
    // }

    reader
        .into_samples::<f32>()
        .map(|x| x.expect("sample"))
        .collect::<Vec<_>>()
}

pub fn convert_integer_to_float_audio(samples: &[i32]) -> Vec<f32> {
    let mut floats = Vec::with_capacity(samples.len());
    for sample in samples {
        floats.push(*sample as f32);
    }
    floats
}
