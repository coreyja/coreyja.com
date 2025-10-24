use cja::{app_state::AppState as _, jobs::Job};
use db::DiscordChannel;
use serde::{Deserialize, Serialize};
use serenity::all::ChannelId;

use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshDiscordChannels;

#[async_trait::async_trait]
impl Job<AppState> for RefreshDiscordChannels {
    const NAME: &'static str = "RefreshDiscordChannels";

    async fn run(&self, state: AppState) -> cja::Result<()> {
        let Some(ref discord) = state.discord else {
            tracing::info!("Discord not configured, skipping channel refresh");
            return Ok(());
        };

        let channels = sqlx::query_as!(DiscordChannel, "SELECT * FROM DiscordChannels")
            .fetch_all(state.db())
            .await?;

        for channel in channels {
            tracing::info!("Refreshing channel: {}", channel.channel_id);

            let channel_id = ChannelId::new(channel.channel_id.parse::<u64>()?);

            let channel = discord.http.get_channel(channel_id).await?;

            let serenity::all::Channel::Guild(channel) = channel else {
                tracing::error!("Channel is a DM Channel");
                continue;
            };

            sqlx::query!(
                "UPDATE DiscordChannels SET channel_name = $1, channel_topic = $2 WHERE channel_id = $3",
                channel.name,
                channel.topic,
                channel.id.to_string()
            )
            .execute(state.db())
            .await?;
        }

        Ok(())
    }
}
