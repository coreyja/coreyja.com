use std::path::PathBuf;

use clap::Args;
use miette::IntoDiagnostic;
use openai::{
    chat::{complete_chat, ChatMessage},
    OpenAiConfig,
};
use srtlib::{Subtitle, Subtitles, Timestamp};
use tokio::{fs::create_dir_all, process::Command};
use tracing::info;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};

#[derive(Debug, Args)]
pub(crate) struct Single {
    #[clap(long)]
    video: PathBuf,
    #[clap(long)]
    date: String,
}

impl Single {
    pub async fn process(&self) -> miette::Result<()> {
        let openai_config = OpenAiConfig::from_env()?;

        // First we need to transcribe the video with Whisper
        let ctx = WhisperContext::new("./models/ggml-base.en.bin").expect("failed to load model");

        create_dir_all("./tmp").await.into_diagnostic()?;

        Command::new("ffmpeg")
            .arg("-i")
            .arg("./tmp/video.mkv")
            .arg("-ac")
            .arg("1")
            .args(["-ar", "16000"])
            .args(["-acodec", "pcm_s16le"])
            .arg("./tmp/audio.wav")
            .output()
            .await
            .into_diagnostic()?;

        info!("Transcribing audio");

        // create a params object
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 5 });
        params.set_n_threads(
            std::thread::available_parallelism()
                .into_diagnostic()?
                .get() as i32,
        );
        let params = params;

        let mut reader = hound::WavReader::open("./tmp/audio.wav").unwrap();
        let audio_data = reader
            .samples::<i16>()
            .collect::<Result<Vec<_>, _>>()
            .into_diagnostic()?;

        // now we can run the model
        let mut state = ctx.create_state().expect("failed to create state");
        state
            .full(
                params,
                &whisper_rs::convert_integer_to_float_audio(&audio_data),
            )
            .expect("failed to run model");

        // fetch the results
        let num_segments = state
            .full_n_segments()
            .expect("failed to get number of segments");

        let mut text_only_buffer = String::new();
        let mut buffer = String::new();
        let mut subs = Subtitles::new();
        for i in 0..num_segments {
            if let Ok(segment) = state.full_get_segment_text(i) {
                let start_timestamp = state
                    .full_get_segment_t0(i)
                    .expect("failed to get segment start timestamp");
                let end_timestamp = state
                    .full_get_segment_t1(i)
                    .expect("failed to get segment end timestamp");

                text_only_buffer.push_str(&segment);
                text_only_buffer.push('\n');

                buffer.push_str(&format!(
                    "[{} - {}]: {}\n",
                    start_timestamp, end_timestamp, segment
                ));

                let start = Timestamp::from_seconds(start_timestamp as i32);
                let end = Timestamp::from_seconds(end_timestamp as i32);
                let sub = Subtitle::new((i + 1) as usize, start, end, segment);
                subs.push(sub);
            } else {
                continue;
            };
        }
        tokio::fs::write("./tmp/transciption.txt", &buffer)
            .await
            .into_diagnostic()?;
        tokio::fs::write("./tmp/transciption_text_only.txt", &text_only_buffer)
            .await
            .into_diagnostic()?;

        subs.write_to_file("./tmp/subtitles.srt", None)
            .into_diagnostic()?;
        info!("Wrote Transcription to Disk");

        let mut summaries: Vec<String> = vec![];
        let lines = text_only_buffer.lines().collect::<Vec<_>>();
        for chunk in lines.chunks(500) {
            let resp = complete_chat(
                &openai_config,
                "gpt-4",
                vec![ChatMessage {
                    role: openai::chat::ChatRole::System,
                    content: format!(
                        "The following is a portion of the transcript of a recorded live stream.
Please summarize the transcript

The summary should be as detailed as possible.
Include any details about the project we are working on and any technologies used or mentioned

{}",
                        chunk.join("\n")
                    ),
                }],
            )
            .await?;

            let summary = resp.content;
            dbg!(&summary);
            summaries.push(summary);
        }

        let full_summary = summaries.join("\n");
        tokio::fs::write("./tmp/summary.txt", &full_summary)
            .await
            .into_diagnostic()?;

        let date = &self.date;
        let resp = complete_chat(
              &openai_config,
              "gpt-4",
              vec![ChatMessage {
                  role: openai::chat::ChatRole::System,
                  content: format!(
                      "The following is a GPT created summary of a single live stream video.
The summary was generated from the audio transcript,
and may have been broken into multiple parts.

The host's name is Corey.
Corey uses he/him pronouns and goes by coreyja online
Recording Date: {date}

Most of the coding is done in Rust.

Please create a Youtube Video Title and Description from the entire input.
Include the title on the first line, followed by a blank line, followed by the description.
Each response should be a single title and description

The titles and descriptions should be written in a way that would make people want to watch the video.
Include the date of the recording in the description of the video
                      
                  
{}",
full_summary
                  ),
              }],
          )
          .await?;

        tokio::fs::write("./tmp/youtube.txt", &resp.content)
            .await
            .into_diagnostic()?;

        Ok(())
    }
}

trait FromSeconds {
    fn from_seconds(seconds: i32) -> Self;
}

impl FromSeconds for Timestamp {
    fn from_seconds(seconds: i32) -> Self {
        let mut t = Timestamp::new(0, 0, 0, 0);
        t.add_seconds(seconds);
        t
    }
}
