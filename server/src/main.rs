#![allow(dead_code)]

use std::{collections::HashMap, fs::OpenOptions, sync::Arc, time::Duration};

use clap::Parser;
use commands::Command;
use miette::{Context, IntoDiagnostic};
use opentelemetry_otlp::WithExportConfig;
use poise::serenity_prelude::{self as serenity};
use posts::{blog::BlogPosts, past_streams::PastStreams, til::TilPosts};
use sentry::ClientInitGuard;
use serde::{Deserialize, Serialize};

use sqlx::{migrate, SqlitePool};
use tokio::try_join;
use tracing::{info, instrument};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{prelude::*, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

use async_trait::async_trait;

pub use miette::Result;

mod discord;
use discord::*;

mod twitch;
use twitch::*;

mod http_server;
use http_server::{pages::blog::md::SyntaxHighlightingContext, *};

mod github;
use github::*;

mod db;
use db::*;

use openai::*;

mod commands;
mod posts;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AppConfig {
    base_url: String,
}

impl AppConfig {
    #[instrument]
    fn from_env() -> Result<Self> {
        Ok(Self {
            base_url: std::env::var("APP_BASE_URL")
                .into_diagnostic()
                .wrap_err("Missing APP_BASE_URL, needed for app launch")?,
        })
    }

    fn app_url(&self, path: &str) -> String {
        if path.starts_with('/') {
            format!("{}{}", self.base_url, path)
        } else {
            format!("{}/{}", self.base_url, path)
        }
    }

    fn home_page(&self) -> String {
        self.base_url.clone()
    }
}

#[derive(Debug, Clone)]
struct AppState {
    twitch: TwitchConfig,
    db_pool: SqlitePool,
    github: GithubConfig,
    open_ai: OpenAiConfig,
    app: AppConfig,
    markdown_to_html_context: SyntaxHighlightingContext,
    blog_posts: Arc<BlogPosts>,
    til_posts: Arc<TilPosts>,
    streams: Arc<PastStreams>,
}

fn setup_sentry() -> Option<ClientInitGuard> {
    let git_commit = std::option_env!("CIRCLE_SHA1");
    let release_name = sentry::release_name!().unwrap_or_else(|| "dev".into());
    let release_name = if let Some(git_commit) = git_commit {
        git_commit.into()
    } else {
        release_name
    };

    if let Ok(sentry_dsn) = std::env::var("SENTRY_DSN") {
        println!("Sentry enabled");

        Some(sentry::init((
            sentry_dsn,
            sentry::ClientOptions {
                traces_sample_rate: 0.0,
                release: Some(release_name),
                ..Default::default()
            },
        )))
    } else {
        println!("Sentry not configured in this environment");

        None
    }
}

fn setup_tracing() -> Result<()> {
    let rust_log =
        std::env::var("RUST_LOG").unwrap_or_else(|_| "warn,server=trace,tower_http=debug".into());

    let env_filter = EnvFilter::builder()
        .parse(&rust_log)
        .into_diagnostic()
        .wrap_err_with(|| miette::miette!("Couldn't create env filter from {}", rust_log))?;

    let opentelemetry_layer = if let Ok(honeycomb_key) = std::env::var("HONEYCOMB_API_KEY") {
        let mut map = HashMap::<String, String>::new();
        map.insert("x-honeycomb-team".to_string(), honeycomb_key);
        map.insert("x-honeycomb-dataset".to_string(), "coreyja.com".to_string());

        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .http()
                    .with_endpoint("https://api.honeycomb.io/v1/traces")
                    .with_timeout(Duration::from_secs(3))
                    .with_headers(map),
            )
            .install_batch(opentelemetry::runtime::Tokio)
            .into_diagnostic()?;

        let opentelemetry_layer = OpenTelemetryLayer::new(tracer);
        println!("Honeycomb layer configured");

        Some(opentelemetry_layer)
    } else {
        println!("Skipping Honeycomb layer");

        None
    };

    let heirarchical = {
        let heirarchical = HierarchicalLayer::default()
            .with_writer(std::io::stdout)
            .with_indent_lines(true)
            .with_indent_amount(2)
            .with_thread_names(true)
            .with_thread_ids(true)
            .with_verbose_exit(true)
            .with_verbose_entry(true)
            .with_targets(true);

        println!("Let's also log to stdout.");

        heirarchical
    };

    Registry::default()
        .with(heirarchical)
        .with(opentelemetry_layer)
        .with(env_filter)
        .try_init()
        .into_diagnostic()?;

    Ok(())
}

#[derive(Parser)]
#[command(author, version, about)]
struct CliArgs {
    #[clap(subcommand)]
    command: Option<Command>,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    let _sentry_guard = setup_sentry();
    setup_tracing()?;

    let cli = CliArgs::parse();
    let command = cli.command.unwrap_or_default();

    command.run().await
}

#[cfg(test)]
mod test {

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn validate() -> miette::Result<()> {
        crate::commands::validate::validate().await
    }
}
