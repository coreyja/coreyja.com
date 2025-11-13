use cja::Result;
use clap::Subcommand;

pub(crate) mod info;
pub(crate) mod validate;

#[derive(Subcommand, Default)]
pub(crate) enum Command {
    #[default]
    Serve,
    Print,
    Validate,
}

impl Command {
    pub(crate) async fn run(&self) -> Result<()> {
        match &self {
            Command::Serve => crate::http_server::cmd::serve().await,
            Command::Print => info::print_info(),
            Command::Validate => validate::validate(),
        }
    }
}
