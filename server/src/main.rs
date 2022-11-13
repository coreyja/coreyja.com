use std::net::SocketAddr;

use axum_macros::debug_handler;
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::try_join;
use tracing::{info, instrument, warn};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Layer, Registry};
use tracing_tree::HierarchicalLayer;

pub use color_eyre::Result;

mod discord;
use discord::*;

mod twitch;
use twitch::*;

mod http_server;
use http_server::*;

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

    let discord_future = run_discord_bot(&twitch_config);
    let axum_future = run_axum(&twitch_config);

    let chatters = get_chatters(&twitch_config).await;
    dbg!(chatters);

    try_join!(discord_future, axum_future)?;

    Ok(())
}
