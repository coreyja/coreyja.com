use std::{fs::OpenOptions, net::SocketAddr};

use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};

use sqlx::{migrate, SqlitePool};
use tokio::try_join;
use tracing::warn;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Layer, Registry};
use tracing_tree::HierarchicalLayer;

pub use color_eyre::Result;

mod discord;
use discord::*;

mod twitch;
use twitch::*;

mod http_server;
use http_server::*;

#[derive(Debug, Clone)]
struct Config {
    twitch: TwitchConfig,
    db_pool: SqlitePool,
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
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let twitch_config = TwitchConfig::from_env()?;

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        let path = std::env::var("DATABASE_PATH");

        if let Ok(p) = &path {
            OpenOptions::new().write(true).create(true).open(p).unwrap();

            format!("sqlite:{}", p)
        } else {
            "sqlite::memory:".to_string()
        }
    });

    let pool = SqlitePool::connect(&database_url).await?;

    let config = Config {
        twitch: twitch_config,
        db_pool: pool,
    };

    migrate!("./migrations/").run(&config.db_pool).await?;

    let discord_future = run_discord_bot(config.clone());
    let axum_future = run_axum(config.clone());

    let chatters = get_chatters(&config.twitch).await;
    dbg!(chatters);

    try_join!(discord_future, axum_future)?;

    Ok(())
}
