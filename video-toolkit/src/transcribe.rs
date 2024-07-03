use aws_sdk_s3 as s3;
use clap::Args;
use futures::TryStreamExt;
use s3::primitives::ByteStream;
use tokio::{io::AsyncWriteExt, process::Command};
use tracing::info;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};

use crate::get_all_objects_for_bucket;

const CACHE_DIR: &str = "./.cache";

#[derive(Debug, Args)]
pub(crate) struct TranscribeVideos {
    #[clap(long)]
    bucket: String,
    #[clap(long)]
    prefix: String,
}

impl TranscribeVideos {
    pub(crate) async fn transcribe_videos(&self) -> color_eyre::Result<()> {
        let config = ::aws_config::load_from_env().await;
        let client = s3::Client::new(&config);

        let ctx = WhisperContext::new("./models/ggml-base.en.bin").expect("failed to load model");

        let objects = get_all_objects_for_bucket(client, &self.bucket, &self.prefix).await?;

        let videos = objects
            .iter()
            .filter(|x| match x.key() {
                Some(k) => k.ends_with(".mkv"),
                None => false,
            })
            .map(Video::from_s3)
            .collect::<Result<Vec<_>, _>>()?;

        let mut big_videos = videos
            .iter()
            .filter(|obj| obj.size > 500_000_000)
            .collect::<Vec<_>>();

        big_videos.sort_by_key(|x| x.size);

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
                        .bucket(&self.bucket)
                        .key(&video.name)
                        .send()
                        .await?;
                    let mut stream = resp.body;
                    // let data = stream
                    //     .collect()
                    //     .await
                    //     .expect("error reading data")
                    //     .into_bytes();

                    let mut fd = cacache::Writer::create(CACHE_DIR, &video.name).await?;
                    while let Some(bytes) = stream.try_next().await? {
                        fd.write_all(&bytes).await?;
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

            info!("Transcribing video");

            info!("Converting video to audio");
            tokio::fs::create_dir_all("./tmp").await?;
            cacache::hard_link(CACHE_DIR, metadata.key, "./tmp/video.mkv").await?;

            Command::new("ffmpeg")
                .arg("-i")
                .arg("./tmp/video.mkv")
                .arg("-ac")
                .arg("1")
                .args(["-ar", "16000"])
                .args(["-acodec", "pcm_s16le"])
                .arg("./tmp/audio.wav")
                .output()
                .await?;

            info!("Transcribing audio");

            // create a params object
            let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 5 });
            params.set_n_threads(std::thread::available_parallelism()?.get() as i32);
            let params = params;

            let mut reader = hound::WavReader::open("./tmp/audio.wav").unwrap();
            let audio_data = reader.samples::<i16>().collect::<Result<Vec<_>, _>>()?;

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
            let mut buffer = String::new();
            for i in 0..num_segments {
                if let Ok(segment) = state.full_get_segment_text(i) {
                    let start_timestamp = state
                        .full_get_segment_t0(i)
                        .expect("failed to get segment start timestamp");
                    let end_timestamp = state
                        .full_get_segment_t1(i)
                        .expect("failed to get segment end timestamp");

                    buffer.push_str(&format!(
                        "[{} - {}]: {}\n",
                        start_timestamp, end_timestamp, segment
                    ));
                } else {
                    continue;
                };
            }

            // Write buffer to tmp/transcription.txt
            std::fs::write("./tmp/transciption.txt", &buffer)?;
            info!("Wrote Transcription to Disk");

            let transcription_path = video.transcription_path().to_str().unwrap().to_string();
            let bytes = buffer.as_bytes();
            let client = s3::Client::new(&config);
            client
                .put_object()
                .bucket(&self.bucket)
                .key(transcription_path)
                .body(ByteStream::from(bytes.to_vec()))
                .send()
                .await?;
            info!("Uploaded transcription to S3");

            info!("Cleaning up tmp dir");
            tokio::fs::remove_dir_all("./tmp").await?;

            info!("Removing video from cache");
            cacache::remove(CACHE_DIR, &video.name).await?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Video {
    name: String,
    size: i64,
}

impl Video {
    fn from_s3(obj: &s3::types::Object) -> color_eyre::Result<Self> {
        Ok(Self {
            name: obj
                .key
                .as_ref()
                .ok_or_else(|| color_eyre::eyre::eyre!("No key in S3 Object"))?
                .to_string(),
            size: obj.size,
        })
    }

    fn has_transcription(&self, all_obects: &[s3::types::Object]) -> bool {
        let transcription_path = self.transcription_path();

        all_obects
            .iter()
            .any(|obj| obj.key == Some(transcription_path.to_str().unwrap().to_string()))
    }

    fn transcription_path(&self) -> std::path::PathBuf {
        let path = std::path::PathBuf::from(&self.name);

        path.with_extension("txt")
    }
}
