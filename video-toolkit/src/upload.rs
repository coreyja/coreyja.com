use std::{fs::File, io::Read};

use clap::Args;
use posts::{past_streams::PastStreams, plain::IntoPlainText};
use tokio::{io::AsyncWriteExt, task::spawn_blocking};
use tracing::info;

use crate::*;

#[derive(Debug, Args)]
pub(crate) struct Upload {
    #[clap(long)]
    bucket: String,
}

impl Upload {
    pub async fn upload(&self) -> Result<()> {
        let config = ::aws_config::load_from_env().await;

        let streams = PastStreams::from_static_dir()?;

        let without_youtube = streams
            .streams
            .iter()
            .filter(|x| x.frontmatter.youtube_url.is_none());

        // info!("Cleaning up tmp dir");
        // tokio::fs::create_dir_all("./tmp").await.into_diagnostic()?;
        // tokio::fs::remove_dir_all("./tmp").await.into_diagnostic()?;

        let hub = google_youtube3::YouTube::new(
            google_youtube3::hyper::Client::builder().build(
                google_youtube3::hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_or_http()
                    .enable_http1()
                    .enable_http2()
                    .build(),
            ),
            std::env::var("COREYJA_YOUTUBE_ACCESS_TOKEN")
                .unwrap()
                .to_string(),
        );

        for stream in without_youtube {
            info!("Uploading: {:?}", stream.frontmatter.title);

            tokio::fs::create_dir_all("./tmp").await.into_diagnostic()?;
            let video_file = tokio::fs::File::open("./tmp/video.mkv").await;

            match video_file {
                Ok(video) => {
                    info!("Using cached video");

                    video
                }
                Err(_) => {
                    info!("Downloading video file");

                    let client = s3::Client::new(&config);
                    let resp = client
                        .get_object()
                        .bucket(&self.bucket)
                        .key(&stream.frontmatter.s3_url)
                        .send()
                        .await
                        .into_diagnostic()?;

                    let stream = resp.body;
                    // let mut fd = cacache::Writer::create(CACHE_DIR, &video_path).await?;
                    let mut fd = tokio::fs::File::create("./tmp/video.mkv")
                        .await
                        .into_diagnostic()?;
                    tokio::io::copy(&mut stream.into_async_read(), &mut fd)
                        .await
                        .into_diagnostic()?;

                    // Data is only committed to the cache after you do `fd.commit()`!
                    // let sri = fd.commit().await?;
                    // dbg!(sri);

                    // cacache::write(CACHE_DIR, &video.name, &data).await?;

                    // cacache::metadata(CACHE_DIR, &video_path).await?.unwrap()
                    fd.flush().await.into_diagnostic()?;

                    fd
                }
            };

            info!("Video Downloaded");
            use google_youtube3::api::{Video, VideoSnippet};

            // As the method needs a request, you would usually fill it with the desired information
            // into the respective structure. Some of the parts shown here might not be applicable !
            // Values shown here are possibly random and not representative !
            let req = Video {
                snippet: Some(VideoSnippet {
                    title: Some(stream.frontmatter.title.to_string()),
                    description: Some(stream.ast.0.plain_text()),
                    ..Default::default()
                }),
                status: Some(google_youtube3::api::VideoStatus {
                    privacy_status: Some("private".to_string()),
                    made_for_kids: Some(false),
                    self_declared_made_for_kids: Some(false),
                    embeddable: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            };
            println!("About to upload");
            let result = hub
                .videos()
                .insert(req)
                .notify_subscribers(false)
                .upload_resumable(
                    File::open("./tmp/video.mkv").into_diagnostic()?,
                    "application/octet-stream".parse().unwrap(),
                )
                .await
                .into_diagnostic()?;
            let video_id = result.1.id.unwrap();
            let youtube_url = format!("https://youtu.be/{}", video_id);

            let path = format!("./past_streams/{}.md", stream.frontmatter.date);
            let content = tokio::fs::read_to_string(&path).await.into_diagnostic()?;

            let mut split = content.split("---");
            let first = split.next().unwrap();
            let second = split.next().unwrap();
            let rest: String = split.collect();

            let new_content = format!(
                "{}---{}youtube_url: \"{}\"\n---{}",
                first, second, youtube_url, rest
            );

            tokio::fs::write(&path, new_content)
                .await
                .into_diagnostic()?;

            info!("Cleaning up tmp dir");
            tokio::fs::create_dir_all("./tmp").await.into_diagnostic()?;
            tokio::fs::remove_dir_all("./tmp").await.into_diagnostic()?;
        }

        Ok(())
    }
}
