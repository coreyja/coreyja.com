use aws_sdk_s3 as s3;
use futures::StreamExt;
use miette::IntoDiagnostic;

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
    let videos = pages
        .into_iter()
        .flat_map(|page| page.contents.unwrap())
        .collect::<Vec<_>>();

    Ok(videos)
}

pub(crate) async fn process_videos() -> Result<(), miette::ErrReport> {
    let config = ::aws_config::load_from_env().await;
    let client = s3::Client::new(&config);

    let objects =
        get_all_objects_for_bucket(client, "coreyja-video-backups", "raw_recordings/2023").await?;

    let big_videos = objects
        .iter()
        .filter(|obj| obj.size() > 1_000_000_000)
        .collect::<Vec<_>>();

    dbg!(big_videos.len());

    let sample = big_videos[0];

    dbg!(sample);

    Ok(())
}
