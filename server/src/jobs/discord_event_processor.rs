use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::state::AppState;
use cja::jobs::Job as JobTrait;
use db::agentic_threads::{Stitch, Thread, ThreadStatus};
use db::discord_threads::DiscordThreadMetadata;

use super::thread_processor::ProcessThreadStep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDiscordEvent {
    pub thread_id: Uuid,
    pub event_type: String,
    pub event_data: serde_json::Value,
}

#[async_trait]
impl JobTrait<AppState> for ProcessDiscordEvent {
    const NAME: &'static str = "ProcessDiscordEvent";

    async fn run(&self, app_state: AppState) -> cja::Result<()> {
        let db = &app_state.db;

        // Get the thread
        tracing::info!("Getting thread by id: {}", self.thread_id);
        let thread = Thread::get_by_id(db, self.thread_id)
            .await?
            .ok_or_else(|| color_eyre::eyre::eyre!("Thread not found"))?;

        // Get the last stitch to maintain stitch chain
        tracing::info!("Getting last stitch for thread: {}", self.thread_id);
        let last_stitch = Stitch::get_last_stitch(db, self.thread_id).await?;

        tracing::info!("Event type: {}", self.event_type);
        match self.event_type.as_str() {
            "message" => {
                // Create a discord_message stitch with the message content
                let message_data = json!({
                    "type": "discord_message",
                    "data": self.event_data,
                    "timestamp": Utc::now().to_rfc3339(),
                });

                let _stitch = Stitch::create_discord_message(
                    db,
                    self.thread_id,
                    last_stitch.as_ref().map(|s| s.stitch_id),
                    message_data,
                )
                .await?;

                // Update thread status to running if it's not already
                if thread.status != ThreadStatus::Running {
                    tracing::info!("Updating thread status to running");
                    Thread::update_status(db, self.thread_id, "running").await?;
                }

                // Update Discord metadata with last message ID
                if let Some(message_id) = self.event_data.get("message_id").and_then(|v| v.as_str())
                {
                    DiscordThreadMetadata::update_last_message_id(
                        db,
                        self.thread_id,
                        message_id.to_string(),
                    )
                    .await?;
                }

                // Enqueue thread processor to handle the message
                ProcessThreadStep {
                    thread_id: self.thread_id,
                }
                .enqueue(app_state.clone(), "Discord message processing".to_string())
                .await?;
            }

            "thread_create" => {
                // Send initial greeting message
                let greeting = "Hello! I'm here to help you in this Discord thread. Feel free to ask me anything!";

                // Create an initial prompt stitch
                let _stitch =
                    Stitch::create_initial_user_message(db, self.thread_id, greeting.to_string())
                        .await?;

                // Update thread status to running
                Thread::update_status(db, self.thread_id, "running").await?;

                // Enqueue thread processor
                ProcessThreadStep {
                    thread_id: self.thread_id,
                }
                .enqueue(
                    app_state.clone(),
                    "Discord thread creation processing".to_string(),
                )
                .await?;
            }

            _ => {
                tracing::warn!("Unknown Discord event type: {}", self.event_type);
            }
        }

        Ok(())
    }
}
