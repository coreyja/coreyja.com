use std::fs::File;

use clap::Args;
use futures::TryStreamExt;
use s3::primitives::ByteStream;
use tokio::io::AsyncWriteExt;
use tracing::info;

use crate::*;

#[derive(Debug, Args)]
pub(crate) struct Upload {
    #[clap(long)]
    bucket: String,
    #[clap(long)]
    prefix: String,
}

const CACHE_DIR: &str = "./.cache";

impl Upload {
    pub async fn upload(&self) -> Result<()> {
        let config = ::aws_config::load_from_env().await;
        let client = s3::Client::new(&config);

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

        let objects = get_all_objects_for_bucket(client, &self.bucket, &self.prefix).await?;

        let mut video_infos = objects
            .iter()
            .filter(|x| match x.key() {
                Some(k) => k.ends_with(".yt_v1.txt"),
                None => false,
            })
            .collect::<Vec<_>>();

        video_infos.sort_by_key(|x| -x.size);

        dbg!(video_infos.len());

        for info in video_infos {
            info!("Creating Youtube Info: {:?}", info.key());

            let key_path = info.key().unwrap();
            let key_path = key_path.strip_suffix(".yt_v1.txt").unwrap();
            let uploaded_path = format!("{}.upload.txt", key_path);
            let video_path = format!("{}.mkv", key_path);

            if objects.iter().any(|x| x.key().unwrap() == uploaded_path) {
                info!("Video already uploaded");
                continue;
            }

            let client = s3::Client::new(&config);
            let resp = client
                .get_object()
                .bucket(&self.bucket)
                .key(info.key().unwrap())
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
            let cached_video = cacache::metadata(CACHE_DIR, info.key().unwrap()).await?;

            let metadata = match cached_video {
                Some(video) => {
                    info!("Using cached video");

                    video
                }
                None => {
                    info!("Downloading video file");

                    let client = s3::Client::new(&config);
                    let resp = client
                        .get_object()
                        .bucket(&self.bucket)
                        .key(info.key().unwrap())
                        .send()
                        .await
                        .into_diagnostic()?;
                    let mut stream = resp.body;
                    // let data = stream
                    //     .collect()
                    //     .await
                    //     .expect("error reading data")
                    //     .into_bytes();

                    let mut fd = cacache::Writer::create(CACHE_DIR, info.key().unwrap()).await?;
                    while let Some(bytes) = stream.try_next().await.into_diagnostic()? {
                        fd.write_all(&bytes).await.into_diagnostic()?;
                    }
                    // for _ in 0..10 {
                    //     fd.write_all(b"very large data")
                    //         .await
                    //         .expect("Failed to write to cache");
                    // }
                    // Data is only committed to the cache after you do `fd.commit()`!
                    let sri = fd.commit().await?;
                    dbg!(sri);

                    // cacache::write(CACHE_DIR, &video.name, &data).await?;

                    cacache::metadata(CACHE_DIR, info.key().unwrap())
                        .await?
                        .unwrap()
                }
            };

            info!("Video Downloaded");

            info!("Uploading");
            tokio::fs::create_dir_all("./tmp").await.into_diagnostic()?;
            cacache::hard_link(CACHE_DIR, metadata.key, "./tmp/video.mkv").await?;

            let mut s = data.split("\n\n");
            let title = s.next().unwrap();
            let description = s.collect::<Vec<_>>().join("\n\n");

            dbg!(&title, &description);

            if !dialoguer::Confirm::new()
                .with_prompt("Should we upload this video?")
                .interact()
                .into_diagnostic()?
            {
                info!("Skipping upload, and recording that we skipped");
                let client = s3::Client::new(&config);
                client
                    .put_object()
                    .bucket(&self.bucket)
                    .key(&uploaded_path)
                    .body(ByteStream::from_static("skipped".as_bytes()))
                    .send()
                    .await
                    .into_diagnostic()?;

                continue;
            }

            use google_youtube3::api::{Video, VideoSnippet};

            // As the method needs a request, you would usually fill it with the desired information
            // into the respective structure. Some of the parts shown here might not be applicable !
            // Values shown here are possibly random and not representative !
            let req = Video {
                snippet: Some(VideoSnippet {
                    title: Some(title.to_string()),
                    description: Some(description.to_string()),
                    ..Default::default()
                }),
                status: Some(google_youtube3::api::VideoStatus {
                    privacy_status: Some("private".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            };

            let result = hub
                .videos()
                .insert(req)
                .stabilize(false)
                .on_behalf_of_content_owner_channel("coreyja")
                .on_behalf_of_content_owner("coreyja")
                .notify_subscribers(false)
                .auto_levels(true)
                .upload(
                    File::open("./tmp/video.mkv").into_diagnostic()?,
                    "application/octet-stream".parse().unwrap(),
                )
                .await
                .into_diagnostic()?;

            let bytes = result.1.id.as_ref().unwrap().as_bytes();
            let client = s3::Client::new(&config);
            client
                .put_object()
                .bucket(&self.bucket)
                .key(&uploaded_path)
                .body(ByteStream::from(bytes.to_vec()))
                .send()
                .await
                .into_diagnostic()?;

            todo!("Upload Video")
        }

        Ok(())
    }
}
