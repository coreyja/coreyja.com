use async_trait::async_trait;
use cja::color_eyre::eyre::Context;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{
    http_server::webhooks::linear::{
        LinearActivityType, LinearAgentActivity, LinearWebhookPayload,
    },
    state::AppState,
};
use cja::jobs::Job as JobTrait;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessLinearWebhook {
    pub payload: LinearWebhookPayload,
}

#[async_trait]
impl JobTrait<AppState> for ProcessLinearWebhook {
    const NAME: &'static str = "ProcessLinearWebhook";

    async fn run(&self, _app_state: AppState) -> cja::Result<()> {
        let payload = &self.payload;

        info!(
            action = payload.action,
            event_type = payload.event_type,
            "Processing Linear webhook in background job"
        );

        match (payload.event_type.as_str(), payload.action.as_str()) {
            ("AgentSession", "created") => {
                info!("Agent session created, acknowledging with thought activity");

                if let Some(webhook_url) = &payload.url {
                    let activity = LinearAgentActivity {
                        activity_type: LinearActivityType::Thought,
                        message: "Processing request...".to_string(),
                    };

                    emit_agent_activity(webhook_url, activity)
                        .await
                        .wrap_err("Failed to emit initial thought activity")?;
                }
            }
            _ => {
                info!(
                    event_type = payload.event_type,
                    action = payload.action,
                    "Unhandled webhook event type"
                );
            }
        }

        Ok(())
    }
}

async fn emit_agent_activity(webhook_url: &str, activity: LinearAgentActivity) -> cja::Result<()> {
    let client = reqwest::Client::new();

    let response = client
        .post(webhook_url)
        .json(&activity)
        .send()
        .await
        .wrap_err("Failed to emit agent activity")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(cja::color_eyre::eyre::eyre!(
            "Failed to emit agent activity: {status} - {body}"
        ));
    }

    Ok(())
}
