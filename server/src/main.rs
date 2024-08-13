#![allow(dead_code)]

use std::sync::Arc;

use cja::setup::{setup_sentry, setup_tracing};
use clap::Parser;
use commands::Command;

use serde::{Deserialize, Serialize};

use tracing::instrument;

pub use cja::Result;

mod twitch;

mod http_server;

mod github;

mod commands;

mod encrypt;

pub mod tracking;

pub mod cron;
pub mod jobs;
pub mod state;
pub(crate) use state::{AppConfig, AppState};

pub(crate) mod google;

#[derive(Parser)]
#[command(author, version, about)]
struct CliArgs {
    #[clap(subcommand)]
    command: Option<Command>,
}

fn main() -> Result<()> {
    let _sentry_guard = setup_sentry();

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()?
        .block_on(async { _main().await })
}

async fn _main() -> Result<()> {
    setup_tracing("server")?;

    let cli = CliArgs::parse();
    let command = cli.command.unwrap_or_default();

    command.run().await
}

#[cfg(test)]
mod test {

    #[test]
    fn validate() -> cja::Result<()> {
        crate::commands::validate::validate()
    }
}
