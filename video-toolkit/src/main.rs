use blogify::Blogify;
use clap::{Parser, Subcommand};
use futures::StreamExt;
use miette::IntoDiagnostic;
use summarize::Summarize;
use tracing_common::setup_tracing;
use transcribe::TranscribeVideos;

use aws_sdk_s3 as s3;
use miette::Result;
use youtubize::Youtubize;

mod blogify;
mod single;
mod summarize;
mod transcribe;
mod youtubize;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct CliArgs {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    TranscribeVideos(TranscribeVideos),
    Summarize(Summarize),
    Youtubize(Youtubize),
    Blogify(Blogify),
    Single(single::Single),
}

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "info");

    setup_tracing("video-toolkit")?;
    let cli = CliArgs::parse();

    match cli.command {
        Command::TranscribeVideos(transcribe_videos) => {
            transcribe_videos.transcribe_videos().await?
        }
        Command::Summarize(s) => s.summarize().await?,
        Command::Youtubize(y) => y.youtubize().await?,
        Command::Blogify(b) => b.blogify().await?,
        Command::Single(s) => s.process().await?,
    }

    Ok(())
}

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
