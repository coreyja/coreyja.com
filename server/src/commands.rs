use clap::Subcommand;
use miette::Result;

use self::video_backlog::TranscribeVideos;

pub(crate) mod info;
pub(crate) mod validate;
pub(crate) mod video_backlog;

#[derive(Subcommand)]
pub(crate) enum Command {
    Serve,
    Print,
    Validate,
    TranscribeVideos(TranscribeVideos),
}

impl Default for Command {
    fn default() -> Self {
        Self::Serve
    }
}

impl Command {
    pub(crate) async fn run(&self) -> Result<()> {
        match &self {
            Command::Serve => crate::http_server::cmd::serve().await,
            Command::Print => info::print_info().await,
            Command::Validate => validate::validate().await,
            Command::TranscribeVideos(cmd) => cmd.transcribe_videos().await,
        }
    }
}
