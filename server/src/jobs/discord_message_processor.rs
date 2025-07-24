use async_trait::async_trait;
use chrono::Utc;
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::state::AppState;
use cja::jobs::Job as JobTrait;
use db::agentic_threads::{Stitch, Thread, ThreadStatus};
use db::discord_threads::DiscordThreadMetadata;
use serenity::builder::CreateThread;

use super::thread_processor::ProcessThreadStep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDiscordMessage {
    pub message: serenity::Message,
}

#[async_trait]
impl JobTrait<AppState> for ProcessDiscordMessage {
    const NAME: &'static str = "ProcessDiscordMessage";

    async fn run(&self, app_state: AppState) -> cja::Result<()> {
        let db = &app_state.db;
        let msg = &self.message;

        // Skip messages from bots
        if msg.author.bot {
            return Ok(());
        }

        // Get the channel to check its type
        let channel = msg.channel_id.to_channel(&app_state.discord).await?;
        let bot_user = app_state.discord.cache.current_user().clone();

        // Check if this is in a thread
        if let Some(guild_channel) = channel.guild() {
            if guild_channel.kind == serenity::ChannelType::PublicThread
                || guild_channel.kind == serenity::ChannelType::PrivateThread
            {
                // Handle existing thread message
                let thread_id = guild_channel.id.to_string();

                // Try to find existing interactive thread
                let existing_discord =
                    DiscordThreadMetadata::find_by_discord_thread_id(db, &thread_id).await?;

                let thread = if let Some(discord_meta) = existing_discord {
                    // Get the associated thread
                    Thread::get_by_id(db, discord_meta.thread_id)
                        .await?
                        .ok_or_else(|| {
                            color_eyre::eyre::eyre!("Thread not found for Discord metadata")
                        })?
                } else {
                    // Get thread name
                    let thread_name = guild_channel.name.clone();

                    // Create new interactive thread
                    let thread = Thread::create_interactive(
                        db,
                        format!("Interactive Discord thread: {thread_name}"),
                    )
                    .await?;

                    // Create Discord metadata
                    let _discord_meta = DiscordThreadMetadata::create(
                        db,
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
                let _updated =
                    DiscordThreadMetadata::add_participant(db, thread.thread_id, &msg.author.tag())
                        .await?;

                // Process the message
                self.process_message_for_thread(app_state, thread.thread_id).await?;
            } else {
                // Check if bot was mentioned in a regular channel
                if msg.mentions.iter().any(|u| u.id == bot_user.id) {
                    // Create a new Discord thread
                    let thread_name = format!("Thread with {}", msg.author.name);
                    let builder = CreateThread::new(&thread_name)
                        .auto_archive_duration(serenity::AutoArchiveDuration::OneDay);
                    let new_thread = guild_channel
                        .create_thread_from_message(&app_state.discord.http, msg.id, builder)
                        .await?;

                    // Create the interactive thread in the app
                    let ai_thread = Thread::create_interactive(
                        db,
                        format!("Interactive Discord thread: {thread_name}"),
                    )
                    .await?;

                    // Create Discord metadata
                    let _discord_meta = DiscordThreadMetadata::create(
                        db,
                        ai_thread.thread_id,
                        new_thread.id.to_string(),
                        guild_channel.id.to_string(),
                        guild_channel.guild_id.to_string(),
                        msg.author.tag(),
                        thread_name,
                    )
                    .await?;

                    // Add the original user as participant
                    let _updated = DiscordThreadMetadata::add_participant(
                        db,
                        ai_thread.thread_id,
                        &msg.author.tag(),
                    )
                    .await?;

                    // Process the original message
                    self.process_message_for_thread(app_state, ai_thread.thread_id)
                        .await?;
                }
            }
        }

        Ok(())
    }
}

impl ProcessDiscordMessage {
    async fn process_message_for_thread(
        &self,
        app_state: AppState,
        thread_id: Uuid,
    ) -> cja::Result<()> {
        let db = &app_state.db;
        let msg = &self.message;

        // Get the thread
        tracing::info!("Getting thread by id: {}", thread_id);
        let thread = Thread::get_by_id(db, thread_id)
            .await?
            .ok_or_else(|| color_eyre::eyre::eyre!("Thread not found"))?;

        // Get the last stitch to maintain stitch chain
        tracing::info!("Getting last stitch for thread: {}", thread_id);
        let last_stitch = Stitch::get_last_stitch(db, thread_id).await?;

        // Create event data
        let event_data = json!({
            "message_id": msg.id.to_string(),
            "author": msg.author.tag(),
            "author_id": msg.author.id.to_string(),
            "author_name": msg.author.name,
            "author_display_name": msg.author.global_name.as_ref().unwrap_or(&msg.author.name),
            "content": msg.content,
            "timestamp": msg.timestamp.to_rfc3339().unwrap_or_else(|| Utc::now().to_rfc3339()),
            "attachments": msg.attachments.iter().map(|a| json!({
                "filename": a.filename,
                "url": a.url,
                "content_type": a.content_type,
            })).collect::<Vec<_>>(),
        });

        // Create a discord_message stitch with the message content
        let message_data = json!({
            "type": "discord_message",
            "data": event_data,
            "timestamp": Utc::now().to_rfc3339(),
        });

        let _stitch = Stitch::create_discord_message(
            db,
            thread_id,
            last_stitch.as_ref().map(|s| s.stitch_id),
            message_data,
        )
        .await?;

        // Update thread status to running if it's not already
        if thread.status != ThreadStatus::Running {
            tracing::info!("Updating thread status to running");
            Thread::update_status(db, thread_id, "running").await?;
        }

        // Update Discord metadata with last message ID
        DiscordThreadMetadata::update_last_message_id(db, thread_id, msg.id.to_string()).await?;

        // Enqueue thread processor to handle the message
        ProcessThreadStep { thread_id }
            .enqueue(app_state.clone(), "Discord message processing".to_string())
            .await?;

        Ok(())
    }
}