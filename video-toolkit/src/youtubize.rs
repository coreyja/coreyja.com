use std::path::PathBuf;

use clap::Args;
use openai::{
    chat::{complete_chat, ChatMessage},
    OpenAiConfig,
};
use s3::primitives::ByteStream;
use tracing::info;

use crate::*;

#[derive(Debug, Args)]
pub(crate) struct Youtubize {
    #[clap(long)]
    bucket: String,
    #[clap(long)]
    prefix: String,
}

impl Youtubize {
    pub async fn youtubize(&self) -> Result<()> {
        let openai_config = OpenAiConfig::from_env()?;

        let config = ::aws_config::load_from_env().await;
        let client = s3::Client::new(&config);

        let objects = get_all_objects_for_bucket(client, &self.bucket, &self.prefix).await?;

        let mut summaries = objects
            .iter()
            .filter(|x| match x.key() {
                Some(k) => k.ends_with(".summary_v14.txt"),
                None => false,
            })
            .collect::<Vec<_>>();

        summaries.sort_by_key(|x| -x.size);

        dbg!(summaries.len());

        for summary in summaries {
            info!("Creating Youtube Info: {:?}", summary.key());

            let key_path = summary.key().unwrap();
            let key_path = key_path.strip_suffix(".summary_v14.txt").unwrap();
            let youtube_path = format!("{}.yt_v2.txt", key_path);

            if objects.iter().any(|x| x.key().unwrap() == youtube_path) {
                info!("Transcript already has youtube info");
                continue;
            }

            let date = {
                let mut p = PathBuf::from(summary.key().unwrap());
                p.set_extension("");
                let date = p.file_name().unwrap().to_str().unwrap();
                let date = date.split(' ').next().unwrap();

                chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap()
            };
            dbg!(&date);

            let client = s3::Client::new(&config);
            let resp = client
                .get_object()
                .bucket(&self.bucket)
                .key(summary.key().unwrap())
                .send()
                .await
                ?;
            let data = resp
                .body
                .collect()
                .await
                .expect("error reading data")
                .into_bytes();
            let data = String::from_utf8(data.to_vec()).expect("invalid utf8");

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
                        data
                    ),
                }],
            )
            .await?;

            let youtube = resp.content;

            dbg!(&youtube);

            let client = s3::Client::new(&config);
            client
                .put_object()
                .bucket(&self.bucket)
                .key(&youtube_path)
                .body(ByteStream::from(youtube.as_bytes().to_vec()))
                .send()
                .await
                ?;

            info!("Uploaded Youtube Info to {:?}", youtube_path);
            // info!("Sleeping for 30 seconds to avoid Rate Limits");
            // tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }

        Ok(())
    }
}
