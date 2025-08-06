use async_trait::async_trait;
use color_eyre::eyre::Context as _;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use tracing::info;

use crate::{
    encrypt::decrypt,
    http_server::webhooks::linear::LinearWebhookPayload,
    linear::{agent::AgentActivityContent, graphql::create_agent_activity},
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

    async fn run(&self, app_state: AppState) -> cja::Result<()> {
        match &self.payload {
            LinearWebhookPayload::AgentSessionEvent(event) => {
                info!(
                    action = event.action,
                    session_id = event.agent_session.id,
                    "Processing Linear agent session webhook"
                );

                match event.action.as_str() {
                    "created" => {
                        info!("Agent session created, acknowledging with thought activity");

                        // Look up the access token by workspace/organization ID
                        let installation = sqlx::query!(
                            r#"
                            SELECT encrypted_access_token
                            FROM linear_installations
                            WHERE external_workspace_id = $1
                            ORDER BY updated_at DESC
                            LIMIT 1
                            "#,
                            event.organization_id
                        )
                        .fetch_optional(&app_state.db)
                        .await?
                        .ok_or_else(|| {
                            cja::color_eyre::eyre::eyre!(
                                "No Linear installation found for workspace {}",
                                event.organization_id
                            )
                        })?;

                        let access_token = decrypt(
                            &installation.encrypted_access_token,
                            &app_state.encrypt_config,
                        )?;

                        create_agent_activity(
                            &access_token,
                            &event.agent_session.id,
                            AgentActivityContent::thought("Processing request..."),
                        )
                        .await
                        .wrap_err("Failed to emit initial thought activity")?;

                        sleep(std::time::Duration::from_secs(5)).await;

                        create_agent_activity(
                            &access_token,
                            &event.agent_session.id,
                            AgentActivityContent::response("All done! I can't do much right now"),
                        )
                        .await
                        .wrap_err("Failed to emit initial thought activity")?;

                        info!("Successfully sent initial thought activity");
                    }
                    "prompted" => {
                        info!("Agent session prompted with user message");

                        // Handle prompted events here when needed
                        // The user has sent a message, available in event.agent_session.agent_activity
                        if let Some(activity) = &event.agent_session.agent_activity {
                            info!(user_message = activity.body, "Received user prompt");
                        }

                        // TODO: Process the user's message and respond appropriately
                    }
                    _ => {
                        info!(action = event.action, "Unhandled agent session action");
                    }
                }
            }
        }

        Ok(())
    }
}
