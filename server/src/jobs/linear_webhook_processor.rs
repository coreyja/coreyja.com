use async_trait::async_trait;
use color_eyre::eyre::Context as _;
use db::agentic_threads::Stitch;
use db::linear_threads::LinearThreadMetadata;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

use crate::{
    agentic_threads::builder::{LinearMetadata, ThreadBuilder},
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

                // Look up the access token first (needed for both actions)
                let installation = sqlx::query!(
                    r#"
                    SELECT encrypted_access_token, external_actor_id
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

                match event.action.as_str() {
                    "created" => {
                        info!("Agent session created, creating new thread");

                        // Check if we already have a thread for this session
                        let existing_thread = LinearThreadMetadata::find_by_session_id(
                            &app_state.db,
                            &event.agent_session.id,
                        )
                        .await?;

                        if existing_thread.is_some() {
                            info!(
                                "Thread already exists for session {}",
                                event.agent_session.id
                            );
                            return Ok(());
                        }

                        // Build initial goal from issue context
                        let goal = if let Some(issue) = &event.agent_session.issue {
                            format!(
                                "Help with Linear issue: {}\n{}",
                                issue.title, issue.description
                            )
                        } else {
                            "Assist with Linear request".to_string()
                        };

                        // // Extract project_id and team_id from issue if available
                        // let (project_id, team_id) = if let Some(issue) = &event.agent_session.issue
                        // {
                        //     let project_id = None;

                        //     let team_id = issue.team_id.clone();

                        //     (project_id, team_id)
                        // } else {
                        //     (None, None)
                        // };
                        let team_id = event
                            .agent_session
                            .issue
                            .as_ref()
                            .map(|i| i.team_id.clone());

                        // Create the thread with Linear metadata
                        let thread = ThreadBuilder::new(app_state.db.clone())
                            .with_goal(goal)
                            .interactive_linear(LinearMetadata {
                                session_id: event.agent_session.id.clone(),
                                workspace_id: event.organization_id.clone(),
                                issue_id: event.agent_session.issue.as_ref().map(|i| i.id.clone()),
                                issue_title: event
                                    .agent_session
                                    .issue
                                    .as_ref()
                                    .map(|i| i.title.clone()),
                                project_id: None,
                                team_id,
                                created_by_user_id: installation
                                    .external_actor_id
                                    .unwrap_or_default(),
                            })
                            .build()
                            .await?;

                        // Create initial prompt stitch with issue context
                        let initial_prompt = if let Some(issue) = &event.agent_session.issue {
                            json!({
                                "issue": {
                                    "id": issue.id,
                                    "title": issue.title,
                                    "description": issue.description,
                                },
                                "context": "Linear agent session created"
                            })
                        } else {
                            json!({
                                "context": "Linear agent session created without issue context"
                            })
                        };

                        Stitch::create_initial_prompt(
                            &app_state.db,
                            thread.thread_id,
                            initial_prompt.to_string(),
                        )
                        .await?;

                        // Update session status to active
                        LinearThreadMetadata::update_session_status(
                            &app_state.db,
                            thread.thread_id,
                            "active",
                        )
                        .await?;

                        // Send initial acknowledgment
                        create_agent_activity(
                            &access_token,
                            &event.agent_session.id,
                            AgentActivityContent::thought("I'm analyzing your request..."),
                        )
                        .await
                        .wrap_err("Failed to emit initial thought activity")?;

                        // TODO: Trigger actual thread processing

                        info!(
                            "Successfully created thread {} for Linear session",
                            thread.thread_id
                        );
                    }
                    "prompted" => {
                        info!("Agent session prompted with user message");

                        // Find existing thread for this session
                        let thread_metadata = LinearThreadMetadata::find_by_session_id(
                            &app_state.db,
                            &event.agent_session.id,
                        )
                        .await?
                        .ok_or_else(|| {
                            cja::color_eyre::eyre::eyre!(
                                "No thread found for Linear session {}",
                                event.agent_session.id
                            )
                        })?;

                        // Handle prompted events here when needed
                        // The user has sent a message, available in event.agent_session.agent_activity
                        if let Some(activity) = &event.agent_session.agent_activity {
                            info!(user_message = activity.body, "Received user prompt");

                            // Create a new stitch for the user message
                            let stitch_data = json!({
                                "message": activity.body,
                                "source": "linear_prompt"
                            });

                            Stitch::create(
                                &app_state.db,
                                thread_metadata.thread_id,
                                "discord_message", // Using this type for now, will add linear_message later
                                stitch_data,
                                None,
                            )
                            .await?;

                            // Update last activity
                            LinearThreadMetadata::update_last_activity(
                                &app_state.db,
                                thread_metadata.thread_id,
                            )
                            .await?;

                            // Send acknowledgment
                            create_agent_activity(
                                &access_token,
                                &event.agent_session.id,
                                AgentActivityContent::thought("Processing your message..."),
                            )
                            .await
                            .wrap_err("Failed to emit thought activity")?;

                            // TODO: Trigger thread processing to generate response
                        }
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
