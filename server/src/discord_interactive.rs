use poise::serenity_prelude::{self as serenity};
use serde_json::json;

use db::agentic_threads::Thread;
use db::discord_threads::DiscordThreadMetadata;
use tracing::instrument;

use crate::jobs::discord_message_processor::ProcessDiscordMessage;
use crate::jobs::discord_thread_create_processor::ProcessDiscordThreadCreate;
use crate::AppState;
use cja::jobs::Job as JobTrait;

pub struct DiscordEventHandler {
    app_state: AppState,
}

impl DiscordEventHandler {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    #[instrument(name = "DiscordEventHandler::handle_message", err, skip(self, msg))]
    pub async fn handle_message(&self, msg: &serenity::Message) -> cja::Result<()> {
        // Create job with the message
        let job_input = ProcessDiscordMessage {
            message: msg.clone(),
        };

        // Enqueue the job
        job_input
            .enqueue(
                self.app_state.clone(),
                "Discord message processing".to_string(),
            )
            .await?;

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

        let job_input = ProcessDiscordThreadCreate {
            thread_id: ai_thread.thread_id,
            thread_data: event_data,
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