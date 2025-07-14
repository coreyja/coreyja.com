use cja::jobs::Job;
use serde::{Deserialize, Serialize};
use serenity::all::CreateMessage;
use serenity::utils::MessageBuilder;

use crate::al::standup::StandupAgent;
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
        let user_id = app_state.standup.discord_user_id.ok_or_else(|| {
            cja::color_eyre::eyre::eyre!("DAILY_MESSAGE_DISCORD_USER_ID not configured")
        })?;

        // Create the AI agent
        let agent = StandupAgent::new(app_state.standup.anthropic_api_key.clone());

        // Generate the standup message using AI with proper Discord mention format
        let message_content = agent.generate_standup_message().await?;

        use serenity::model::prelude::*;

        let user_mention_id = UserId::new(user_id);
        let message = MessageBuilder::new()
            .mention(&user_mention_id)
            .push("\n\n")
            .push(message_content)
            .build();
        let create_message = CreateMessage::new().content(message);
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