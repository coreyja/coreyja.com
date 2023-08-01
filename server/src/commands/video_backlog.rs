use aws_sdk_s3 as s3;
use futures::{StreamExt, TryStreamExt};
use miette::IntoDiagnostic;
use tokio::{io::AsyncWriteExt, process::Command};
use tracing::info;

const CACHE_DIR: &str = "./.cache";

async fn get_all_objects_for_bucket(
    client: s3::Client,
    bucket: &str,
    prefix: &str,
) -> Result<Vec<s3::types::Object>, miette::Report> {
    let resp = client
        .list_objects_v2()
        .bucket(bucket)
        .prefix(prefix)
        .into_paginator()
        .send()
        .collect::<Vec<_>>()
        .await;
    let pages = resp
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .into_diagnostic()?;
    let objects = pages
        .into_iter()
        .map(|page| {
            page.contents
                .ok_or_else(|| miette::miette!("No contents in page"))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    Ok(objects)
}

pub(crate) async fn process_videos() -> Result<(), miette::ErrReport> {
    let config = ::aws_config::load_from_env().await;
    let client = s3::Client::new(&config);

    let objects =
        get_all_objects_for_bucket(client, "coreyja-video-backups", "raw_recordings/2023").await?;

    let videos = objects
        .iter()
        .filter(|x| match x.key() {
            Some(k) => k.ends_with(".mkv"),
            None => false,
        })
        .map(Video::from_s3)
        .collect::<Result<Vec<_>, _>>()?;

    let big_videos = videos
        .iter()
        .filter(|obj| obj.size > 1_000_000_000)
        .collect::<Vec<_>>();

    dbg!(big_videos.len());

    for video in big_videos {
        info!("Transcribing Video: {:?}", video.name);

        if video.has_transcription(&objects) {
            info!("Video already has transcription");
            continue;
        }

        let cached_video = cacache::metadata(CACHE_DIR, &video.name).await?;

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
                    .bucket("coreyja-video-backups")
                    .key(&video.name)
                    .send()
                    .await
                    .into_diagnostic()?;
                let mut stream = resp.body;
                // let data = stream
                //     .collect()
                //     .await
                //     .expect("error reading data")
                //     .into_bytes();

                let mut fd = cacache::Writer::create(CACHE_DIR, &video.name).await?;
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

                cacache::metadata(CACHE_DIR, &video.name).await?.unwrap()
            }
        };

        dbg!(&metadata);
        info!("Transcribing video");

        info!("Converting video to audio");
        tokio::fs::create_dir_all("./tmp").await.into_diagnostic()?;
        cacache::hard_link(CACHE_DIR, metadata.key, "./tmp/video.mkv").await?;

        Command::new("ffmpeg")
            .arg("-i")
            .arg("./tmp/video.mkv")
            .arg("-ac")
            .arg("1")
            .arg("./tmp/audio.wav")
            .args(["-ar", "8000"])
            .args(["-acodec", "pcm_s16le"])
            .output()
            .await
            .into_diagnostic()?;

        todo!("Transcribe audio")
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct Video {
    name: String,
    size: i64,
}

impl Video {
    fn from_s3(obj: &s3::types::Object) -> Result<Self, miette::Report> {
        Ok(Self {
            name: obj
                .key
                .as_ref()
                .ok_or_else(|| miette::miette!("No key in S3 Object"))?
                .to_string(),
            size: obj.size,
        })
    }

    fn has_transcription(&self, all_obects: &[s3::types::Object]) -> bool {
        let path = std::path::PathBuf::from(&self.name);
        let transcription_path = path.with_extension("srt");

        all_obects
            .iter()
            .any(|obj| obj.key == Some(transcription_path.to_str().unwrap().to_string()))
    }
}
