use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serenity::all::{CreateMessage, MessageBuilder};

use crate::{
    al::tools::{ThreadContext, Tool},
    AppState,
};

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
    pub user_id: Vec<u64>,
    pub message: String,
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
            .send_message(&self.app_state.discord, create_message)
            .await
            .map_err(|e| cja::color_eyre::eyre::eyre!("Failed to send Discord message: {}", e))?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct CompleteThread {
    app_state: AppState,
}

impl CompleteThread {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CompleteThreadInput {
    pub reason: String,
}

#[async_trait::async_trait]
impl Tool for CompleteThread {
    const NAME: &'static str = "complete_thread";
    const DESCRIPTION: &'static str =
        "Mark the current thread as completed. This will end the conversation and mark the thread status as 'completed'. Use this tool when you've finished your work";

    type ToolInput = CompleteThreadInput;
    type ToolOutput = ();

    async fn run(
        &self,
        input: Self::ToolInput,
        context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        use db::agentic_threads::Thread;
        use serde_json::json;

        // Mark the thread as complete with the reason
        Thread::complete(
            &self.app_state.db,
            context.thread_id,
            json!({
                "reason": input.reason,
                "completed_at": chrono::Utc::now().to_rfc3339()
            }),
        )
        .await?;

        Ok(())
    }
}
