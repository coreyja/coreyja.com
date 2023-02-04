use std::{collections::HashMap, fs::OpenOptions, net::SocketAddr, sync::Arc, time::Duration};

use color_eyre::eyre::Context;
use opentelemetry_otlp::WithExportConfig;
use poise::serenity_prelude::{self as serenity, CacheAndHttp, ChannelId, Color};
use reqwest::Client;
use rss::Channel;
use sentry::ClientInitGuard;
use serde::{Deserialize, Serialize};

use sqlx::{migrate, SqlitePool};
use tokio::try_join;
use tracing::info;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{prelude::*, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

use async_trait::async_trait;

pub use color_eyre::Result;

mod discord;
use discord::*;

mod twitch;
use twitch::*;

mod http_server;
use http_server::*;

mod github;
use github::*;

mod db;
use db::*;

mod my_rss;
use my_rss::*;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AppConfig {
    base_url: String,
}

impl AppConfig {
    fn from_env() -> Result<Self> {
        Ok(Self {
            base_url: std::env::var("APP_BASE_URL")
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
}

#[derive(Debug, Clone)]
struct Config {
    twitch: TwitchConfig,
    db_pool: SqlitePool,
    github: GithubConfig,
    rss: RssConfig,
    app: AppConfig,
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
    let env_filter = EnvFilter::from_default_env();

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
            .install_batch(opentelemetry::runtime::Tokio)?;

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

        println!("Let's also log to stdout");

        Some(heirarchical)
    };

    Registry::default()
        .with(heirarchical)
        .with(opentelemetry_layer)
        .with(env_filter)
        .try_init()?;

    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    let _sentry_guard = setup_sentry();
    setup_tracing()?;

    let app_config = AppConfig::from_env()?;
    let twitch_config = TwitchConfig::from_env()?;
    let github_config = GithubConfig::from_env()?;
    let rss_config = RssConfig::from_env()?;

    let database_url: String = std::env::var("DATABASE_URL").or_else(|_| -> Result<String> {
        let path = std::env::var("DATABASE_PATH");

        Ok(if let Ok(p) = &path {
            OpenOptions::new().write(true).create(true).open(p)?;

            format!("sqlite:{}", p)
        } else {
            "sqlite::memory:".to_string()
        })
    })?;

    let pool = SqlitePool::connect(&database_url).await?;

    let config = Config {
        twitch: twitch_config,
        db_pool: pool,
        github: github_config,
        app: app_config,
        rss: rss_config,
    };

    info!("About to run migrations (if any to apply)");
    migrate!("./migrations/").run(&config.db_pool).await?;

    let discord_bot = build_discord_bot(config.clone()).await?;

    let http_and_cache = discord_bot.client().cache_and_http.clone();

    info!("Spawning Tasks");
    let discord_future = tokio::spawn(discord_bot.start());
    let axum_future = tokio::spawn(run_axum(config.clone()));
    let rss_future = tokio::spawn(run_rss(config.clone(), http_and_cache.clone()));
    info!("Tasks Spawned");

    let (discord_result, axum_result, run_rss_result) =
        try_join!(discord_future, axum_future, rss_future)?;

    discord_result?;
    axum_result?;
    run_rss_result?;

    info!("Main Returning");

    Ok(())
}

#[allow(dead_code)]
async fn run_log_chatters_loop(config: Config) -> Result<()> {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

    loop {
        interval.tick().await;

        log_chatters(&config).await?;
    }
}

async fn log_chatters(config: &Config) -> Result<()> {
    let chatters = get_chatters(&config.twitch).await?;

    let chat_log_record = sqlx::query!("INSERT INTO ChatterLogRecord DEFAULT VALUES RETURNING id")
        .fetch_one(&config.db_pool)
        .await?;

    for chatter in chatters.data {
        sqlx::query!(
            "INSERT INTO ChatterLogs (chatters_log_id, name) VALUES (?, ?)",
            chat_log_record.id,
            chatter.user_login
        )
        .execute(&config.db_pool)
        .await?;
    }

    Ok(())
}
