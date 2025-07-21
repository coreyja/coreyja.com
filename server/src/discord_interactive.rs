use std::sync::Arc;

use poise::serenity_prelude::{self as serenity};
use serde_json::json;
use sqlx::PgPool;

use db::agentic_threads::Thread;
use db::discord_threads::DiscordThreadMetadata;

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
            let existing_discord =
                DiscordThreadMetadata::find_by_discord_thread_id(&self.db, &thread_id).await?;

            let thread = if let Some(discord_meta) = existing_discord {
                // Get the associated thread
                Thread::get_by_id(&self.db, discord_meta.thread_id)
                    .await?
                    .ok_or_else(|| {
                        color_eyre::eyre::eyre!("Thread not found for Discord metadata")
                    })?
            } else {
                // Get thread name
                let thread_name = msg
                    .channel(self.discord_client.as_ref())
                    .await?
                    .guild()
                    .map_or_else(|| "Discord Thread".to_string(), |c| c.name.clone());

                // Create new interactive thread
                let thread = Thread::create_interactive(
                    &self.db,
                    format!("Interactive Discord thread: {thread_name}"),
                )
                .await?;

                // Create Discord metadata
                let _discord_meta = DiscordThreadMetadata::create(
                    &self.db,
                    thread.thread_id,
                    thread_id.clone(),
                    msg.channel_id.to_string(),
                    msg.guild_id.map(|id| id.to_string()).unwrap_or_default(),
                    msg.author.tag(),
                    thread_name,
                )
                .await?;

                thread
            };

            // Add participant if new
            let _updated = DiscordThreadMetadata::add_participant(
                &self.db,
                thread.thread_id,
                &msg.author.tag(),
            )
            .await?;

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

        // Create the interactive thread
        let ai_thread = Thread::create_interactive(
            &self.db,
            format!("Interactive Discord thread: {}", thread.name),
        )
        .await?;

        // Create Discord metadata
        let _discord_meta = DiscordThreadMetadata::create(
            &self.db,
            ai_thread.thread_id,
            thread.id.to_string(),
            thread
                .parent_id
                .map(|id| id.to_string())
                .unwrap_or_default(),
            thread.guild_id.to_string(),
            thread.owner_id.map(|id| id.to_string()).unwrap_or_default(),
            thread.name.clone(),
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
