use clap::{Parser, Subcommand};
use tracing_common::setup_tracing;
use transcribe::TranscribeVideos;

mod summarize;
mod transcribe;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct CliArgs {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    TranscribeVideos(TranscribeVideos),
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    std::env::set_var("RUST_LOG", "info");

    setup_tracing()?;
    let cli = CliArgs::parse();

    match cli.command {
        Command::TranscribeVideos(transcribe_videos) => {
            transcribe_videos.transcribe_videos().await?
        }
    }

    Ok(())
}
