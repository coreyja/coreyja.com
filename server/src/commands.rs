use clap::Subcommand;
use miette::Result;

pub(crate) mod info;
pub(crate) mod validate;
pub(crate) mod video_backlog;

#[derive(Subcommand)]
pub(crate) enum Command {
    Serve,
    Print,
    Validate,
    VideoBacklog,
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
            Command::VideoBacklog => video_backlog::process_videos().await,
        }
    }
}
