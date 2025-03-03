use cja::Result;
use clap::Subcommand;

pub(crate) mod info;
pub(crate) mod seed_skeets;
pub(crate) mod validate;

#[derive(Subcommand)]
pub(crate) enum Command {
    Serve,
    Print,
    Validate,
    SeedSkeets,
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
            Command::Print => info::print_info(),
            Command::Validate => validate::validate(),
            Command::SeedSkeets => seed_skeets::seed_skeets().await,
        }
    }
}
