use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serenity::all::{CreateMessage, MessageBuilder};

use crate::{
    al::tools::{ThreadContext, Tool},
    AppState,
};
use db::discord_threads::DiscordThreadMetadata;

#[derive(Clone, Debug)]
pub struct SendDiscordMessage;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DiscordInput {
    pub channel_id: u64,
    pub user_id: Vec<u64>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SendToThreadInput {
    pub message: String,
    pub reply_to_message_id: Option<String>,
}

#[async_trait::async_trait]
impl Tool for SendDiscordMessage {
    const NAME: &'static str = "send_discord_message";
    const DESCRIPTION: &'static str = r#"
    Send a message to a Discord channel. The message and channel id are required. And the list of users to tag is optional. YOU MUST INCLUDE A MESSAGE TO SEND

    Example:
    ```json
    {
        "channel_id": 1234567890,
        "user_id": [1234567890],
        "message": "Hello, world!"
    }
    ```

    Example Multiple Users:
    ```json
    {
        "channel_id": 1234567890,
        "user_id": [1234567890, 1234567891],
        "message": "Hello, world!"
    }
    ```
    "#;

    type ToolInput = DiscordInput;
    type ToolOutput = ();

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        _context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        use serenity::model::prelude::*;

        let mut message = MessageBuilder::new();

        for user_id in input.user_id {
            let user_mention_id = UserId::new(user_id);
            message.mention(&user_mention_id).push("\n\n");
        }

        message.push(input.message);

        let create_message = CreateMessage::new().content(message.build());
        let discord_channel_id = ChannelId::new(input.channel_id);

        // Send the message
        discord_channel_id
            .send_message(&app_state.discord, create_message)
            .await
            .map_err(|e| cja::color_eyre::eyre::eyre!("Failed to send Discord message: {}", e))?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct SendDiscordThreadMessage;

impl SendDiscordThreadMessage {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Tool for SendDiscordThreadMessage {
    const NAME: &'static str = "send_discord_thread_message";
    const DESCRIPTION: &'static str = r#"
    Send a message to the current Discord thread. This tool automatically determines the correct thread from the context.
    
    Example:
    ```json
    {
        "message": "Hello! I'm here to help.",
        "reply_to_message_id": null
    }
    ```
    
    Example with reply:
    ```json
    {
        "message": "That's a great question! Let me explain...",
        "reply_to_message_id": "1234567890"
    }
    ```
    "#;

    type ToolInput = SendToThreadInput;
    type ToolOutput = String;

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        use serenity::model::prelude::*;

        // Get Discord metadata for this thread
        let discord_meta =
            DiscordThreadMetadata::find_by_thread_id(&app_state.db, context.thread.thread_id)
                .await?
                .ok_or_else(|| {
                    cja::color_eyre::eyre::eyre!("Discord metadata not found for this thread")
                })?;

        let discord_thread_id = &discord_meta.discord_thread_id;

        let channel_id = ChannelId::from(
            discord_thread_id
                .parse::<u64>()
                .map_err(|_| cja::color_eyre::eyre::eyre!("Invalid Discord thread ID"))?,
        );

        // Create the message
        let mut create_message = CreateMessage::new().content(&input.message);

        // Add reply reference if provided
        if let Some(reply_to) = &input.reply_to_message_id {
            let message_id = MessageId::from(
                reply_to
                    .parse::<u64>()
                    .map_err(|_| cja::color_eyre::eyre::eyre!("Invalid message ID"))?,
            );
            create_message = create_message.reference_message((channel_id, message_id));
        }

        // Send the message
        let sent_message = channel_id
            .send_message(&app_state.discord, create_message)
            .await
            .map_err(|e| cja::color_eyre::eyre::eyre!("Failed to send Discord message: {}", e))?;

        Ok(sent_message.id.to_string())
    }
}
