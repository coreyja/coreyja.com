use chrono::{Timelike, Utc};
use chrono_tz::US::Eastern;
use cja::jobs::Job;
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, CreateMessage};

use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandupMessage;

#[async_trait::async_trait]
impl Job<AppState> for StandupMessage {
    const NAME: &'static str = "StandupMessage";

    async fn run(&self, app_state: AppState) -> cja::Result<()> {
        // Get current time in Eastern timezone
        let now_utc = Utc::now();
        let now_eastern = now_utc.with_timezone(&Eastern);

        // Check if we're in the 7-8am Eastern time window
        let hour = now_eastern.hour();
        if hour != 7 {
            tracing::debug!(
                "Skipping standup message - current hour is {} Eastern (need hour 7)",
                hour
            );
            return Ok(());
        }

        // Get the channel ID from app state config
        let channel_id = app_state.standup.discord_channel_id.ok_or_else(|| {
            cja::color_eyre::eyre::eyre!("DAILY_MESSAGE_DISCORD_CHANNEL_ID not configured")
        })?;

        // Create the standup message with Discord mention
        let message_content = format!(
            "🌅 Good morning <@coreyja>! It's {} Eastern Time.\n\nTime for standup!",
            now_eastern.format("%A, %B %d, %Y at %I:%M %p")
        );

        let create_message = CreateMessage::new().content(&message_content);
        let discord_channel_id = ChannelId::new(channel_id);

        // Send the message
        discord_channel_id
            .send_message(&app_state.discord, create_message)
            .await
            .map_err(|e| cja::color_eyre::eyre::eyre!("Failed to send Discord message: {}", e))?;

        tracing::info!(
            "Standup message sent successfully to channel {}",
            channel_id
        );

        Ok(())
    }
}
