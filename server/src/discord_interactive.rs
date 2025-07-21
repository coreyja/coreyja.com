use std::sync::Arc;

use poise::serenity_prelude::{self as serenity};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;

use db::agentic_threads::Thread;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordMetadata {
    pub channel_id: String,
    pub thread_id: String,
    pub guild_id: String,
    pub webhook_url: Option<String>,
    pub last_message_id: Option<String>,
    pub participants: Vec<String>,
    pub created_by: String,
    pub thread_name: String,
}

use crate::jobs::discord_event_processor::ProcessDiscordEvent;
pub type ProcessDiscordEventInput = ProcessDiscordEvent;

pub struct DiscordEventHandler {
    db: Arc<PgPool>,
    discord_client: Arc<serenity::Http>,
}

impl DiscordEventHandler {
    pub fn new(db: Arc<PgPool>, discord_client: Arc<serenity::Http>) -> Self {
        Self { db, discord_client }
    }

    pub async fn handle_message(&self, msg: &serenity::Message) -> cja::Result<()> {
        // Skip messages from bots
        if msg.author.bot {
            return Ok(());
        }

        // Check if this is in a thread
        if let Some(thread_id) = msg
            .channel_id
            .to_channel(self.discord_client.as_ref())
            .await?
            .guild()
            .and_then(|c| {
                if c.kind == serenity::ChannelType::PublicThread
                    || c.kind == serenity::ChannelType::PrivateThread
                {
                    Some(c.id.to_string())
                } else {
                    None
                }
            })
        {
            // Try to find existing interactive thread
            let existing_thread = Thread::find_by_discord_thread_id(&self.db, &thread_id).await?;

            let thread = if let Some(thread) = existing_thread {
                thread
            } else {
                // Create new interactive thread
                let metadata = DiscordMetadata {
                    channel_id: msg.channel_id.to_string(),
                    thread_id: thread_id.clone(),
                    guild_id: msg.guild_id.map(|id| id.to_string()).unwrap_or_default(),
                    webhook_url: None,
                    last_message_id: Some(msg.id.to_string()),
                    participants: vec![msg.author.tag()],
                    created_by: msg.author.tag(),
                    thread_name: msg
                        .channel(self.discord_client.as_ref())
                        .await?
                        .guild()
                        .map_or_else(|| "Discord Thread".to_string(), |c| c.name.clone()),
                };

                let metadata_json = json!({
                    "discord": metadata
                });

                Thread::create_interactive(
                    &self.db,
                    format!("Interactive Discord thread: {}", metadata.thread_name),
                    metadata_json,
                )
                .await?
            };

            // Create job to process the message
            let event_data = json!({
                "message_id": msg.id.to_string(),
                "author": msg.author.tag(),
                "content": msg.content,
                "timestamp": msg.timestamp.to_rfc3339(),
                "attachments": msg.attachments.iter().map(|a| json!({
                    "filename": a.filename,
                    "url": a.url,
                    "content_type": a.content_type,
                })).collect::<Vec<_>>(),
            });

            let _job_input = ProcessDiscordEventInput {
                thread_id: thread.thread_id,
                event_type: "message".to_string(),
                event_data,
            };

            // Enqueue the job
            // Enqueue job requires AppState, not just db
            // This is a limitation we'll need to work around
            tracing::warn!("TODO: Need AppState to enqueue job from event handler");
        }

        Ok(())
    }

    pub async fn handle_thread_create(&self, thread: &serenity::GuildChannel) -> cja::Result<()> {
        // Check if bot was mentioned or if this is a new thread we should join
        if thread.kind != serenity::ChannelType::PublicThread
            && thread.kind != serenity::ChannelType::PrivateThread
        {
            return Ok(());
        }

        // Get thread starter message if available
        let _starter_message = thread
            .id
            .to_channel(self.discord_client.as_ref())
            .await?
            .guild()
            .and_then(|c| c.thread_metadata);

        // Create metadata for the new thread
        let metadata = DiscordMetadata {
            channel_id: thread
                .parent_id
                .map(|id| id.to_string())
                .unwrap_or_default(),
            thread_id: thread.id.to_string(),
            guild_id: thread.guild_id.to_string(),
            webhook_url: None,
            last_message_id: None,
            participants: vec![],
            created_by: thread.owner_id.map(|id| id.to_string()).unwrap_or_default(),
            thread_name: thread.name.clone(),
        };

        let metadata_json = json!({
            "discord": metadata
        });

        // Create the interactive thread
        let ai_thread = Thread::create_interactive(
            &self.db,
            format!("Interactive Discord thread: {}", thread.name),
            metadata_json,
        )
        .await?;

        // Send initial greeting
        let event_data = json!({
            "action": "send_greeting",
            "thread_name": thread.name,
        });

        let _job_input = ProcessDiscordEventInput {
            thread_id: ai_thread.thread_id,
            event_type: "thread_create".to_string(),
            event_data,
        };

        // Enqueue job requires AppState, not just db
        // This is a limitation we'll need to work around
        tracing::warn!("TODO: Need AppState to enqueue job from event handler");

        Ok(())
    }
}
