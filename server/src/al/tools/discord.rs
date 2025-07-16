use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serenity::all::{CreateMessage, MessageBuilder};

use crate::{al::tools::Tool, AppState};

#[derive(Clone, Debug)]
pub struct SendDiscordMessage {
    app_state: AppState,
}

impl SendDiscordMessage {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DiscordInput {
    pub channel_id: u64,
    pub user_id: Option<u64>,
    pub content: String,
}

#[async_trait::async_trait]
impl Tool for SendDiscordMessage {
    const NAME: &'static str = "send_discord_message";
    const DESCRIPTION: &'static str = "Send a message to a Discord channel";

    type ToolInput = DiscordInput;
    type ToolOutput = ();

    async fn run(&self, input: Self::ToolInput) -> cja::Result<Self::ToolOutput> {
        use serenity::model::prelude::*;

        let mut message = MessageBuilder::new();

        if let Some(user_id) = input.user_id {
            let user_mention_id = UserId::new(user_id);
            message.mention(&user_mention_id).push("\n\n");
        }

        message.push(input.content);

        let create_message = CreateMessage::new().content(message.build());
        let discord_channel_id = ChannelId::new(input.channel_id);

        // Send the message
        discord_channel_id
            .send_message(&self.app_state.discord, create_message)
            .await
            .map_err(|e| cja::color_eyre::eyre::eyre!("Failed to send Discord message: {}", e))?;

        Ok(())
    }
}
