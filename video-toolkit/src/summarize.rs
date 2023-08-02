use std::ops::Index;

use clap::Args;
use openai::{
    chat::{complete_chat, ChatMessage},
    OpenAiConfig,
};
use regex::Regex;
use s3::primitives::ByteStream;
use tracing::info;

use crate::*;

#[derive(Debug, Args)]
pub(crate) struct Summarize {
    #[clap(long)]
    bucket: String,
    #[clap(long)]
    prefix: String,
}

impl Summarize {
    pub async fn summarize(&self) -> Result<()> {
        let openai_config = OpenAiConfig::from_env()?;

        let config = ::aws_config::load_from_env().await;
        let client = s3::Client::new(&config);

        let objects = get_all_objects_for_bucket(client, &self.bucket, &self.prefix).await?;

        let mut transcripts = objects
            .iter()
            .filter(|x| match x.key() {
                Some(k) => k.ends_with(".txt") && !k.contains("summary"),
                None => false,
            })
            .collect::<Vec<_>>();

        transcripts.sort_by_key(|x| x.size);

        dbg!(transcripts.len());

        for transcript in transcripts {
            info!("Summarizing Transcript: {:?}", transcript.key());

            let key_path = transcript.key().unwrap();
            let key_path = key_path.strip_suffix(".txt").unwrap();
            let summary_path = format!("{}.summary_v5.txt", key_path);

            if objects.iter().any(|x| x.key().unwrap() == summary_path) {
                info!("Transcript already has summary");
                continue;
            }

            let client = s3::Client::new(&config);
            let resp = client
                .get_object()
                .bucket(&self.bucket)
                .key(transcript.key().unwrap())
                .send()
                .await
                .into_diagnostic()?;
            let data = resp
                .body
                .collect()
                .await
                .expect("error reading data")
                .into_bytes();
            let data = String::from_utf8(data.to_vec()).expect("invalid utf8");

            let re = Regex::new(r"\[.*\]:(.*)").unwrap();
            let data: String = data
                .lines()
                .map(|x| {
                    let m = re.captures(x).unwrap();
                    m.index(1).trim().to_string()
                })
                .collect::<Vec<_>>()
                .join("\n");

            let resp = complete_chat(
                &openai_config,
                vec![ChatMessage {
                    role: openai::chat::ChatRole::System,
                    content: format!(
                        "The following is a transcript of a recorded live stream.
                    Please summarize the content of the livestream.
                    Do not respond with information about the timestamps.

                    Format your summary as a Youtube Video description and title.
                    On the first line title the video.
                    The next line should be blank.
                    On the third line, write the description.
                    The description should be a paragraph or two long and draw in the reader
                    The title should also be attention grabbing

                    Include any details about the project we are working on and any technologies used or mentioned
                    
                    {}",
                        data
                    ),
                }],
            )
            .await?;

            let summary = resp.content;
            dbg!(&summary);

            let client = s3::Client::new(&config);
            client
                .put_object()
                .bucket(&self.bucket)
                .key(&summary_path)
                .body(ByteStream::from(summary.as_bytes().to_vec()))
                .send()
                .await
                .into_diagnostic()?;

            info!("Uploaded summary to {:?}", summary_path);
            // info!("Sleeping for 1 min to avoid Rate Limits");
            // tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }

        Ok(())
    }
}
