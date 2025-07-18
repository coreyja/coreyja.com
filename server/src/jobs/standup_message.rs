use cja::jobs::Job;
use serde::{Deserialize, Serialize};

use crate::al::standup::StandupAgent;
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandupMessage;

#[async_trait::async_trait]
impl Job<AppState> for StandupMessage {
    const NAME: &'static str = "StandupMessage";

    async fn run(&self, app_state: AppState) -> cja::Result<()> {
        // Create the AI agent
        let agent = StandupAgent::new(app_state.clone());

        agent.generate_standup_message().await?;

        tracing::info!("Standup Agent ran successfully");

        Ok(())
    }
}
