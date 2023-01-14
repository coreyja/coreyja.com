use std::{fs::OpenOptions, net::SocketAddr, sync::Arc};

use color_eyre::eyre::Context;
use poise::serenity_prelude::{self as serenity, CacheAndHttp, ChannelId, Color};
use reqwest::Client;
use rss::Channel;
use serde::{Deserialize, Serialize};

use sqlx::{migrate, SqlitePool};
use tokio::try_join;
use tracing::info;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Layer, Registry};
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
}

#[derive(Debug, Clone)]
struct Config {
    twitch: TwitchConfig,
    db_pool: SqlitePool,
    github: GithubConfig,
    rss: RssConfig,
    app: AppConfig,
}

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::from_default_env();
    let subscriber = Registry::default().with(
        HierarchicalLayer::new(2)
            .with_ansi(true)
            .with_verbose_entry(true)
            .with_verbose_exit(true)
            .with_bracketed_fields(true)
            .with_filter(filter),
    );
    tracing::subscriber::set_global_default(subscriber)?;

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

    migrate!("./migrations/").run(&config.db_pool).await?;

    let discord_bot = build_discord_bot(config.clone()).await?;

    let http_and_cache = discord_bot.client().cache_and_http.clone();

    let discord_future = tokio::spawn(discord_bot.start());
    let axum_future = tokio::spawn(run_axum(config.clone()));

    let rss_future = tokio::spawn(run_rss(config.clone(), http_and_cache.clone()));

    let (discord_result, axum_result, run_rss_result) =
        try_join!(discord_future, axum_future, rss_future)?;

    discord_result?;
    axum_result?;
    run_rss_result?;

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
