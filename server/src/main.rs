#![allow(dead_code)]

use std::{collections::HashMap, sync::Arc, time::Duration};

use ::posts::{blog::BlogPosts, past_streams::PastStreams, til::TilPosts};
use clap::Parser;
use commands::Command;
use miette::{Context, IntoDiagnostic};
use opentelemetry_otlp::WithExportConfig;
use posts::projects::Projects;
use sentry::ClientInitGuard;
use serde::{Deserialize, Serialize};

use tracing::{info, instrument};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{prelude::*, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

pub use miette::Result;

mod twitch;
use twitch::*;

mod http_server;
use http_server::{pages::blog::md::SyntaxHighlightingContext, *};

mod github;
use github::*;

use openai::*;

mod commands;

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
struct VersionInfo {
    git_commit: &'static str,
    rustc_version: &'static str,
}

impl VersionInfo {
    fn from_env() -> Self {
        Self {
            git_commit: env!("VERGEN_GIT_SHA"),
            rustc_version: env!("VERGEN_RUSTC_SEMVER"),
        }
    }
}

#[derive(Debug, Clone)]
struct AppState {
    twitch: TwitchConfig,
    github: GithubConfig,
    open_ai: OpenAiConfig,
    app: AppConfig,
    markdown_to_html_context: SyntaxHighlightingContext,
    blog_posts: Arc<BlogPosts>,
    til_posts: Arc<TilPosts>,
    streams: Arc<PastStreams>,
    projects: Arc<Projects>,
    versions: VersionInfo,
}

fn setup_sentry() -> Option<ClientInitGuard> {
    let git_commit: Option<std::borrow::Cow<_>> = option_env!("VERGEN_GIT_SHA").map(|x| x.into());
    let release_name =
        git_commit.unwrap_or_else(|| sentry::release_name!().unwrap_or_else(|| "dev".into()));

    if let Ok(sentry_dsn) = std::env::var("SENTRY_DSN") {
        println!("Sentry enabled");

        Some(sentry::init((
            sentry_dsn,
            sentry::ClientOptions {
                traces_sample_rate: 0.5,
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
        .with(sentry_tracing::layer())
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

fn main() -> Result<()> {
    let _sentry_guard = setup_sentry();

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .into_diagnostic()?
        .block_on(async { _main().await })
}

async fn _main() -> Result<()> {
    setup_tracing()?;

    let cli = CliArgs::parse();
    let command = cli.command.unwrap_or_default();

    command.run().await
}

#[cfg(test)]
mod test {

    #[tokio::test]
    async fn validate() -> miette::Result<()> {
        crate::commands::validate::validate().await
    }
}
