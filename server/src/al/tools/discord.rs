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
    #[serde(default)]
    pub continue_processing: bool,
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
    
    By default, after sending a message, the agent will wait for a user response before continuing.
    Set continue_processing to true if you want to perform additional actions after sending the message.
    
    Example (default - wait for response):
    ```json
    {
        "message": "Hello! I'm here to help.",
        "reply_to_message_id": null,
        "continue_processing": false
    }
    ```
    
    Example with reply:
    ```json
    {
        "message": "That's a great question! Let me explain...",
        "reply_to_message_id": "1234567890",
        "continue_processing": false
    }
    ```
    
    Example (continue processing):
    ```json
    {
        "message": "Let me search for that information...",
        "reply_to_message_id": null,
        "continue_processing": true
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

        // Only set thread status to waiting if continue_processing is false
        // This allows the agent to perform additional actions if needed
        if !input.continue_processing {
            db::agentic_threads::Thread::update_status(
                &app_state.db,
                context.thread.thread_id,
                "waiting",
            )
            .await?;
        }

        Ok(sent_message.id.to_string())
    }
}

#[derive(Clone, Debug)]
pub struct ListenToThread;

impl ListenToThread {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListenInput {}

#[async_trait::async_trait]
impl Tool for ListenToThread {
    const NAME: &'static str = "listen_to_thread";
    const DESCRIPTION: &'static str = "
    Listen to the current Discord thread without sending a message. Use this when you want to wait for user input or observe the conversation without immediately responding.
    
    This tool is useful when:
    - The user asks you to wait or listen
    - You want to give the user time to provide more context
    - Users are chatting and your input isn't needed
    - The conversation naturally calls for a pause
    ";

    type ToolInput = ListenInput;
    type ToolOutput = String;

    async fn run(
        &self,
        _input: Self::ToolInput,
        app_state: AppState,
        context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        use db::agentic_threads::Thread;

        // Get Discord metadata for this thread
        let discord_meta =
            DiscordThreadMetadata::find_by_thread_id(&app_state.db, context.thread.thread_id)
                .await?
                .ok_or_else(|| {
                    cja::color_eyre::eyre::eyre!("Discord metadata not found for this thread")
                })?;

        // Log that we're listening
        tracing::info!(
            "Listen tool activated for Discord thread: {}",
            discord_meta.discord_thread_id,
        );

        // Set thread status to waiting to prevent re-enqueueing
        Thread::update_status(&app_state.db, context.thread.thread_id, "waiting").await?;

        // The actual listening happens by not sending a message
        // The thread processor will wait for the next user message
        Ok("Listening to thread...".to_string())
    }
}

#[derive(Clone, Debug)]
pub struct ReactToMessage;

impl ReactToMessage {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReactInput {
    pub message_id: String,
    pub emoji: String,
}

#[async_trait::async_trait]
impl Tool for ReactToMessage {
    const NAME: &'static str = "react_to_message";
    const DESCRIPTION: &'static str = r#"
    Add a reaction emoji to a Discord message in the current thread.
    
    The emoji can be:
    - A unicode emoji like "👍", "❤️", "🎉", etc.
    - A custom Discord emoji in the format "<:name:id>" or "<a:name:id>" for animated emojis
    
    Example with unicode emoji:
    ```json
    {
        "message_id": "1234567890",
        "emoji": "👍"
    }
    ```
    
    Example with custom emoji:
    ```json
    {
        "message_id": "1234567890", 
        "emoji": "<:customname:123456789012345678>"
    }
    ```
    "#;

    type ToolInput = ReactInput;
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

        let message_id = MessageId::from(
            input
                .message_id
                .parse::<u64>()
                .map_err(|_| cja::color_eyre::eyre::eyre!("Invalid message ID"))?,
        );

        // Parse the emoji
        let reaction_type = if input.emoji.starts_with("<:") || input.emoji.starts_with("<a:") {
            // Custom emoji format: <:name:id> or <a:name:id>
            // Extract the emoji ID from the format
            let parts: Vec<&str> = input
                .emoji
                .trim_matches(&['<', '>', ':'][..])
                .split(':')
                .collect();
            if parts.len() >= 2 {
                if let Ok(emoji_id) = parts[parts.len() - 1].parse::<u64>() {
                    serenity::all::ReactionType::Custom {
                        animated: input.emoji.starts_with("<a:"),
                        id: serenity::all::EmojiId::from(emoji_id),
                        name: Some(parts[parts.len() - 2].to_string()),
                    }
                } else {
                    return Err(cja::color_eyre::eyre::eyre!("Invalid custom emoji format"));
                }
            } else {
                return Err(cja::color_eyre::eyre::eyre!("Invalid custom emoji format"));
            }
        } else {
            // Unicode emoji
            serenity::all::ReactionType::Unicode(input.emoji.clone())
        };

        // Add the reaction
        app_state
            .discord
            .http
            .create_reaction(channel_id, message_id, &reaction_type)
            .await
            .map_err(|e| cja::color_eyre::eyre::eyre!("Failed to add reaction: {}", e))?;

        Ok(format!("Added {} reaction to message", input.emoji))
    }
}

#[derive(Clone, Debug)]
pub struct ListServerEmojis;

impl ListServerEmojis {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListEmojisInput {
    pub include_animated: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmojiInfo {
    pub name: String,
    pub id: String,
    pub animated: bool,
    pub format: String,
}

#[async_trait::async_trait]
impl Tool for ListServerEmojis {
    const NAME: &'static str = "list_server_emojis";
    const DESCRIPTION: &'static str = r#"
    List all custom emojis available in the Discord server.
    
    Returns a list of custom emojis with their names, IDs, and formatted strings that can be used with the react_to_message tool.
    
    Example:
    ```json
    {
        "include_animated": true
    }
    ```
    "#;

    type ToolInput = ListEmojisInput;
    type ToolOutput = Vec<EmojiInfo>;

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

        // Parse guild ID
        let guild_id = GuildId::from(
            discord_meta
                .guild_id
                .parse::<u64>()
                .map_err(|_| cja::color_eyre::eyre::eyre!("Invalid guild ID"))?,
        );

        // Get the guild
        let guild = guild_id
            .to_partial_guild(&app_state.discord.http)
            .await
            .map_err(|e| cja::color_eyre::eyre::eyre!("Failed to get guild: {}", e))?;

        // Get emojis from the guild
        let emojis = guild
            .emojis(&app_state.discord.http)
            .await
            .map_err(|e| cja::color_eyre::eyre::eyre!("Failed to get emojis: {}", e))?;

        // Filter and format emojis
        let include_animated = input.include_animated.unwrap_or(true);
        let emoji_list: Vec<EmojiInfo> = emojis
            .into_iter()
            .filter(|emoji| include_animated || !emoji.animated)
            .map(|emoji| {
                let format = if emoji.animated {
                    format!("<a:{}:{}>", emoji.name, emoji.id)
                } else {
                    format!("<:{}:{}>", emoji.name, emoji.id)
                };

                EmojiInfo {
                    name: emoji.name.clone(),
                    id: emoji.id.to_string(),
                    animated: emoji.animated,
                    format,
                }
            })
            .collect();

        Ok(emoji_list)
    }
}
