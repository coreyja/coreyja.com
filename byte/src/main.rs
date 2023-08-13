use std::{
    fs::File,
    io::BufWriter,
    sync::{Arc, Mutex},
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, Sample,
};
pub use miette::Result;
use miette::{Context, IntoDiagnostic};
use tracing_common::setup_tracing;

const PREFERRED_MIC_NAME: &str = "Samson G-Track Pro";

#[tokio::main]
async fn main() -> Result<()> {
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
