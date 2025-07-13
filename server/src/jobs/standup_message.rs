use chrono::Utc;
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
        // Get the channel ID from app state config
        let channel_id = app_state.standup.discord_channel_id.ok_or_else(|| {
            cja::color_eyre::eyre::eyre!("DAILY_MESSAGE_DISCORD_CHANNEL_ID not configured")
        })?;

        let now_eastern = Utc::now().with_timezone(&Eastern);

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
