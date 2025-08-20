use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;
use cja::jobs::Job as JobTrait;
use db::agentic_threads::{Stitch, Thread};

use super::thread_processor::ProcessThreadStep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDiscordThreadCreate {
    pub thread_id: Uuid,
    pub thread_data: serde_json::Value,
}

#[async_trait]
impl JobTrait<AppState> for ProcessDiscordThreadCreate {
    const NAME: &'static str = "ProcessDiscordThreadCreate";

    async fn run(&self, app_state: AppState) -> cja::Result<()> {
        let db = &app_state.db;

        // Send initial greeting message
        let greeting =
            "Hello! I'm here to help you in this Discord thread. Feel free to ask me anything!";

        // Create an initial prompt stitch
        let _stitch = Stitch::create_initial_user_message(db, self.thread_id, greeting).await?;

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

        Ok(())
    }
}
