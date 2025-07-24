use poise::serenity_prelude::{self as serenity};

use db::agentic_threads::Thread;
use db::discord_threads::DiscordThreadMetadata;
use tracing::instrument;

use crate::jobs::discord_message_processor::ProcessDiscordMessage;
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
