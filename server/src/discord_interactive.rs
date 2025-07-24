use poise::serenity_prelude::{self as serenity};
use serde_json::json;

use db::agentic_threads::Thread;
use db::discord_threads::DiscordThreadMetadata;
use serenity::builder::CreateThread;
use tracing::instrument;

use crate::jobs::discord_event_processor::ProcessDiscordEvent;
use crate::AppState;
use cja::jobs::Job as JobTrait;
pub type ProcessDiscordEventInput = ProcessDiscordEvent;

pub struct DiscordEventHandler {
    app_state: AppState,
}

impl DiscordEventHandler {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    #[instrument(name = "DiscordEventHandler::handle_message", err, skip(self, msg))]
    pub async fn handle_message(&self, msg: &serenity::Message) -> cja::Result<()> {
        // Skip messages from bots
        if msg.author.bot {
            return Ok(());
        }

        // Get the channel to check its type
        let channel = msg.channel_id.to_channel(&self.app_state.discord).await?;

        // Get the current bot user
        let bot_user = self.app_state.discord.cache.current_user().clone();

        // Check if this is in a thread
        if let Some(guild_channel) = channel.guild() {
            if guild_channel.kind == serenity::ChannelType::PublicThread
                || guild_channel.kind == serenity::ChannelType::PrivateThread
            {
                // Handle existing thread message
                let thread_id = guild_channel.id.to_string();

                // Try to find existing interactive thread
                let existing_discord = DiscordThreadMetadata::find_by_discord_thread_id(
                    &self.app_state.db,
                    &thread_id,
                )
                .await?;

                let thread = if let Some(discord_meta) = existing_discord {
                    // Get the associated thread
                    Thread::get_by_id(&self.app_state.db, discord_meta.thread_id)
                        .await?
                        .ok_or_else(|| {
                            color_eyre::eyre::eyre!("Thread not found for Discord metadata")
                        })?
                } else {
                    // Get thread name
                    let thread_name = guild_channel.name.clone();

                    // Create new interactive thread
                    let thread = Thread::create_interactive(
                        &self.app_state.db,
                        format!("Interactive Discord thread: {thread_name}"),
                    )
                    .await?;

                    // Create Discord metadata
                    let _discord_meta = DiscordThreadMetadata::create(
                        &self.app_state.db,
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
                    &self.app_state.db,
                    thread.thread_id,
                    &msg.author.tag(),
                )
                .await?;

                // Create job to process the message
                let event_data = json!({
                    "message_id": msg.id.to_string(),
                    "author": msg.author.tag(),
                    "author_id": msg.author.id.to_string(),
                    "author_name": msg.author.name,
                    "author_display_name": msg.author.global_name.as_ref().unwrap_or(&msg.author.name),
                    "content": msg.content,
                    "timestamp": msg.timestamp.to_rfc3339(),
                    "attachments": msg.attachments.iter().map(|a| json!({
                        "filename": a.filename,
                        "url": a.url,
                        "content_type": a.content_type,
                    })).collect::<Vec<_>>(),
                });

                let job_input = ProcessDiscordEventInput {
                    thread_id: thread.thread_id,
                    event_type: "message".to_string(),
                    event_data,
                };

                job_input
                    .enqueue(
                        self.app_state.clone(),
                        "Discord message processing".to_string(),
                    )
                    .await?;
            } else {
                // Check if bot was mentioned in a regular channel
                if msg.mentions.iter().any(|u| u.id == bot_user.id) {
                    // Create a new Discord thread
                    let thread_name = format!("Thread with {}", msg.author.name);
                    let builder = CreateThread::new(&thread_name)
                        .auto_archive_duration(serenity::AutoArchiveDuration::OneDay);
                    let new_thread = guild_channel
                        .create_thread_from_message(&self.app_state.discord.http, msg.id, builder)
                        .await?;

                    // Create the interactive thread in the app
                    let ai_thread = Thread::create_interactive(
                        &self.app_state.db,
                        format!("Interactive Discord thread: {}", thread_name),
                    )
                    .await?;

                    // Create Discord metadata
                    let _discord_meta = DiscordThreadMetadata::create(
                        &self.app_state.db,
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
                        &self.app_state.db,
                        ai_thread.thread_id,
                        &msg.author.tag(),
                    )
                    .await?;

                    // Process the original message as the first message in the thread
                    let event_data = json!({
                        "message_id": msg.id.to_string(),
                        "author": msg.author.tag(),
                        "author_id": msg.author.id.to_string(),
                        "author_name": msg.author.name,
                        "author_display_name": msg.author.global_name.as_ref().unwrap_or(&msg.author.name),
                        "content": msg.content,
                        "timestamp": msg.timestamp.to_rfc3339(),
                        "attachments": msg.attachments.iter().map(|a| json!({
                            "filename": a.filename,
                            "url": a.url,
                            "content_type": a.content_type,
                        })).collect::<Vec<_>>(),
                        "original_message_id": msg.id.to_string(), // Track the original message for responding
                        "original_channel_id": guild_channel.id.to_string(),
                    });

                    let job_input = ProcessDiscordEventInput {
                        thread_id: ai_thread.thread_id,
                        event_type: "message".to_string(), // Process as a regular message, not thread_create
                        event_data,
                    };

                    job_input
                        .enqueue(
                            self.app_state.clone(),
                            "Discord message processing from bot mention".to_string(),
                        )
                        .await?;
                }
            }
        }

        Ok(())
    }

    #[instrument(
        name = "DiscordEventHandler::handle_thread_create",
        err,
        skip(self, thread)
    )]
    pub async fn handle_thread_create(&self, thread: &serenity::GuildChannel) -> cja::Result<()> {
        // Check if bot was mentioned or if this is a new thread we should join
        if thread.kind != serenity::ChannelType::PublicThread
            && thread.kind != serenity::ChannelType::PrivateThread
        {
            return Ok(());
        }

        // Check if this Discord thread already exists (created from bot mention)
        let existing_discord = DiscordThreadMetadata::find_by_discord_thread_id(
            &self.app_state.db,
            &thread.id.to_string(),
        )
        .await?;

        // If thread already exists, skip creation (it was created from bot mention)
        if existing_discord.is_some() {
            tracing::info!(
                discord_thread_id = thread.id.to_string(),
                "Discord thread already exists, skipping duplicate creation"
            );
            return Ok(());
        }

        // Get thread starter message if available
        let _starter_message = thread
            .id
            .to_channel(self.app_state.discord.http.as_ref())
            .await?
            .guild()
            .and_then(|c| c.thread_metadata);

        // Create the interactive thread
        let ai_thread = Thread::create_interactive(
            &self.app_state.db,
            format!("Interactive Discord thread: {}", thread.name),
        )
        .await?;

        // Create Discord metadata
        let _discord_meta = DiscordThreadMetadata::create(
            &self.app_state.db,
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

        // For manually created threads (not from bot mention), send a greeting
        let event_data = json!({
            "action": "send_greeting",
            "thread_name": thread.name,
        });

        let job_input = ProcessDiscordEventInput {
            thread_id: ai_thread.thread_id,
            event_type: "thread_create".to_string(),
            event_data,
        };

        // Enqueue the job
        job_input
            .enqueue(
                self.app_state.clone(),
                "Discord thread creation processing".to_string(),
            )
            .await?;

        Ok(())
    }

    #[instrument(
        name = "DiscordEventHandler::handle_thread_update",
        err,
        skip(self, thread)
    )]
    pub async fn handle_thread_update(&self, thread: &serenity::GuildChannel) -> cja::Result<()> {
        // Only process thread channels
        if thread.kind != serenity::ChannelType::PublicThread
            && thread.kind != serenity::ChannelType::PrivateThread
        {
            return Ok(());
        }

        // Check if thread is archived
        if let Some(metadata) = &thread.thread_metadata {
            if metadata.archived {
                // Find the thread in our database
                let discord_meta = DiscordThreadMetadata::find_by_discord_thread_id(
                    &self.app_state.db,
                    &thread.id.to_string(),
                )
                .await?;

                if let Some(discord_meta) = discord_meta {
                    // Update the thread status to completed
                    Thread::update_status(&self.app_state.db, discord_meta.thread_id, "completed")
                        .await?;

                    tracing::info!(
                        thread_id = discord_meta.thread_id.to_string(),
                        discord_thread_id = thread.id.to_string(),
                        "Discord thread archived, marking as completed"
                    );
                }
            }
        }

        Ok(())
    }

    #[instrument(
        name = "DiscordEventHandler::handle_thread_delete",
        err,
        skip(self, thread)
    )]
    pub async fn handle_thread_delete(
        &self,
        thread: &serenity::PartialGuildChannel,
    ) -> cja::Result<()> {
        // Find the thread in our database
        let discord_meta = DiscordThreadMetadata::find_by_discord_thread_id(
            &self.app_state.db,
            &thread.id.to_string(),
        )
        .await?;

        if let Some(discord_meta) = discord_meta {
            // Update the thread status to completed
            Thread::update_status(&self.app_state.db, discord_meta.thread_id, "completed").await?;

            tracing::info!(
                thread_id = discord_meta.thread_id.to_string(),
                discord_thread_id = thread.id.to_string(),
                "Discord thread deleted, marking as completed"
            );
        }

        Ok(())
    }
}
