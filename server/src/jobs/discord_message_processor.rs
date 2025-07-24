use async_trait::async_trait;
use chrono::Utc;
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
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
            if Self::is_thread_channel(&guild_channel) {
                self.handle_thread_message(&app_state, &guild_channel)
                    .await?;
            } else if Self::is_bot_mentioned(msg, &bot_user) {
                self.handle_non_thread_bot_mention(&app_state, &guild_channel)
                    .await?;
            }
        }

        Ok(())
    }
}

impl ProcessDiscordMessage {
    fn is_thread_channel(channel: &serenity::GuildChannel) -> bool {
        channel.kind == serenity::ChannelType::PublicThread
            || channel.kind == serenity::ChannelType::PrivateThread
    }

    fn is_bot_mentioned(msg: &serenity::Message, bot_user: &serenity::CurrentUser) -> bool {
        msg.mentions.iter().any(|u| u.id == bot_user.id)
    }

    async fn handle_thread_message(
        &self,
        app_state: &AppState,
        guild_channel: &serenity::GuildChannel,
    ) -> cja::Result<()> {
        let db = &app_state.db;
        let thread_id = guild_channel.id.to_string();

        // Try to find existing interactive thread
        let existing_discord =
            DiscordThreadMetadata::find_by_discord_thread_id(db, &thread_id).await?;

        let thread = if let Some(discord_meta) = existing_discord {
            self.get_existing_thread(db, discord_meta.thread_id).await?
        } else if Self::is_bot_mentioned(&self.message, &app_state.discord.cache.current_user()) {
            self.create_new_thread_from_discord(db, &thread_id, guild_channel)
                .await?
        } else {
            return Ok(());
        };

        // Add participant if new
        self.add_participant(db, thread.thread_id).await?;

        // Process the message
        self.process_message_for_thread(app_state.clone(), thread.thread_id)
            .await?;

        Ok(())
    }

    async fn handle_non_thread_bot_mention(
        &self,
        app_state: &AppState,
        guild_channel: &serenity::GuildChannel,
    ) -> cja::Result<()> {
        let db = &app_state.db;
        let msg = &self.message;

        // Create a new Discord thread
        let thread_name = format!("Thread with {}", msg.author.name);
        let new_discord_thread = self
            .create_discord_thread(app_state, guild_channel, &thread_name)
            .await?;

        // Create the interactive thread in the app
        let ai_thread = self.create_interactive_thread(db, &thread_name).await?;

        // Create Discord metadata
        self.create_discord_metadata(
            db,
            ai_thread.thread_id,
            new_discord_thread.id.to_string(),
            guild_channel.id.to_string(),
            guild_channel.guild_id.to_string(),
            thread_name,
        )
        .await?;

        // Add the original user as participant
        self.add_participant(db, ai_thread.thread_id).await?;

        // Process the original message
        self.process_message_for_thread(app_state.clone(), ai_thread.thread_id)
            .await?;

        Ok(())
    }

    async fn get_existing_thread(&self, db: &PgPool, thread_id: Uuid) -> cja::Result<Thread> {
        Thread::get_by_id(db, thread_id)
            .await?
            .ok_or_else(|| color_eyre::eyre::eyre!("Thread not found for Discord metadata"))
    }

    async fn create_new_thread_from_discord(
        &self,
        db: &PgPool,
        discord_thread_id: &str,
        guild_channel: &serenity::GuildChannel,
    ) -> cja::Result<Thread> {
        let thread_name = guild_channel.name.clone();
        let msg = &self.message;

        // Create new interactive thread
        let thread = self.create_interactive_thread(db, &thread_name).await?;

        // Create Discord metadata
        self.create_discord_metadata(
            db,
            thread.thread_id,
            discord_thread_id.to_string(),
            msg.channel_id.to_string(),
            msg.guild_id.map(|id| id.to_string()).unwrap_or_default(),
            thread_name,
        )
        .await?;

        Ok(thread)
    }

    async fn create_interactive_thread(
        &self,
        db: &PgPool,
        thread_name: &str,
    ) -> cja::Result<Thread> {
        Thread::create_interactive(db, format!("Interactive Discord thread: {thread_name}")).await
    }

    async fn create_discord_thread(
        &self,
        app_state: &AppState,
        guild_channel: &serenity::GuildChannel,
        thread_name: &str,
    ) -> cja::Result<serenity::GuildChannel> {
        let msg = &self.message;
        let builder = CreateThread::new(thread_name)
            .auto_archive_duration(serenity::AutoArchiveDuration::OneDay);

        guild_channel
            .create_thread_from_message(&app_state.discord.http, msg.id, builder)
            .await
            .map_err(Into::into)
    }

    async fn create_discord_metadata(
        &self,
        db: &PgPool,
        thread_id: Uuid,
        discord_thread_id: String,
        channel_id: String,
        guild_id: String,
        thread_name: String,
    ) -> cja::Result<()> {
        let msg = &self.message;

        DiscordThreadMetadata::create(
            db,
            thread_id,
            discord_thread_id,
            channel_id,
            guild_id,
            msg.author.tag(),
            thread_name,
        )
        .await?;

        Ok(())
    }

    async fn add_participant(&self, db: &PgPool, thread_id: Uuid) -> cja::Result<()> {
        let msg = &self.message;
        DiscordThreadMetadata::add_participant(db, thread_id, &msg.author.tag()).await?;
        Ok(())
    }

    async fn process_message_for_thread(
        &self,
        app_state: AppState,
        thread_id: Uuid,
    ) -> cja::Result<()> {
        let db = &app_state.db;

        // Get the thread
        tracing::info!("Getting thread by id: {}", thread_id);
        let thread = self.get_existing_thread(db, thread_id).await?;

        // Create and save the message stitch
        self.create_message_stitch(db, thread_id).await?;

        // Update thread status if needed
        self.update_thread_status(db, &thread).await?;

        // Update last message ID
        self.update_last_message_id(db, thread_id).await?;

        // Enqueue thread processor
        self.enqueue_thread_processor(app_state, thread_id).await?;

        Ok(())
    }

    async fn create_message_stitch(&self, db: &PgPool, thread_id: Uuid) -> cja::Result<()> {
        // Get the last stitch to maintain stitch chain
        tracing::info!("Getting last stitch for thread: {}", thread_id);
        let last_stitch = Stitch::get_last_stitch(db, thread_id).await?;

        // Create event data
        let event_data = serde_json::to_value(&self.message)?;

        // Create a discord_message stitch with the message content
        let message_data = json!({
            "type": "discord_message",
            "data": event_data,
            "timestamp": Utc::now().to_rfc3339(),
        });

        Stitch::create_discord_message(
            db,
            thread_id,
            last_stitch.as_ref().map(|s| s.stitch_id),
            message_data,
        )
        .await?;

        Ok(())
    }

    async fn update_thread_status(&self, db: &PgPool, thread: &Thread) -> cja::Result<()> {
        if thread.status != ThreadStatus::Running {
            tracing::info!("Updating thread status to running");
            Thread::update_status(db, thread.thread_id, "running").await?;
        }
        Ok(())
    }

    async fn update_last_message_id(&self, db: &PgPool, thread_id: Uuid) -> cja::Result<()> {
        let msg = &self.message;
        DiscordThreadMetadata::update_last_message_id(db, thread_id, msg.id.to_string()).await?;
        Ok(())
    }

    async fn enqueue_thread_processor(
        &self,
        app_state: AppState,
        thread_id: Uuid,
    ) -> cja::Result<()> {
        ProcessThreadStep { thread_id }
            .enqueue(app_state, "Discord message processing".to_string())
            .await?;
        Ok(())
    }
}
