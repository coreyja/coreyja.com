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
                Some(k) => {
                    k.ends_with(".txt")
                        && !k.contains("summary")
                        && !k.contains("yt")
                        && !k.contains("upload")
                }
                None => false,
            })
            .collect::<Vec<_>>();

        transcripts.sort_by_key(|x| -x.size);

        dbg!(transcripts.len());

        for transcript in transcripts {
            info!("Summarizing Transcript: {:?}", transcript.key());

            let key_path = transcript.key().unwrap();
            let key_path = key_path.strip_suffix(".txt").unwrap();
            let summary_path = format!("{}.summary_v14.txt", key_path);

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
            let lines = data
                .lines()
                .map(|x| {
                    let m = re.captures(x).unwrap();
                    m.index(1).trim().to_string()
                })
                .collect::<Vec<_>>();

            let mut summaries: Vec<String> = vec![];
            for chunk in lines.chunks(500) {
                let resp = complete_chat(
                    &openai_config,
                    "gpt-3.5-turbo-16k",
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

            // let summary = if summaries.len() == 1 {
            //     summaries[0].to_string()
            // } else {
            //     let combined = summaries.join("\n");
            //     let resp = complete_chat(
            //       &openai_config,
            //       "gpt-3.5-turbo-16k",
            //       vec![ChatMessage {
            //           role: openai::chat::ChatRole::System,
            //           content: format!(
            //               "The following is a series of summaries of parts of a transcript of a recorded live stream.
            //               Combine the summaries into a summary for the entire stream.
            //               There should be a single summary for the entire stream.

            //               The summary should be as detailed as possible. Keep as many details from the original transcript as possible.

            //           {}",
            //           combined
            //           ),
            //       }],
            //   )
            //   .await?;

            //     resp.content
            // };

            let summary = summaries.join("\n\n");

            println!("\n\n");
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
