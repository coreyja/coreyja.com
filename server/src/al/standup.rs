use std::sync::Arc;

use chrono::Utc;
use chrono_tz::US::Eastern;
use db::agentic_threads::{Stitch, Thread};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::types::Uuid;
use tokio::sync::Mutex;

use crate::{
    al::tools::{
        discord::{DoneTool, SendDiscordMessage},
        ThreadContext, ToolBag,
    },
    AppState,
};

#[derive(Debug, thiserror::Error)]
#[error("Standup loop error")]
enum StandupLoopError {
    #[error("Aborted loop")]
    AbortedLoop,
    #[error("Error: {0}")]
    Other(#[from] cja::color_eyre::Report),
}

impl From<reqwest::Error> for StandupLoopError {
    fn from(err: reqwest::Error) -> Self {
        Self::Other(err.into())
    }
}

impl From<serde_json::Error> for StandupLoopError {
    fn from(err: serde_json::Error) -> Self {
        Self::Other(err.into())
    }
}

impl From<sqlx::Error> for StandupLoopError {
    fn from(err: sqlx::Error) -> Self {
        Self::Other(err.into())
    }
}

#[derive(Debug, Serialize)]
pub struct AnthropicTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    tools: Vec<AnthropicTool>,
    tool_choice: Option<ToolChoice>,
}

#[derive(Debug, Serialize)]
struct ToolChoice {
    r#type: String,
}

#[derive(Debug, Serialize, Clone)]
struct Message {
    role: String,
    content: Vec<Content>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AnthropicResponse {
    content: Vec<Content>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Content {
    #[serde(rename = "text")]
    Text(TextContent),
    #[serde(rename = "tool_use")]
    ToolUse(ToolUseContent),
    #[serde(rename = "tool_result")]
    ToolResult(ToolResult),
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TextContent {
    pub text: String,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ToolUseContent {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ToolResult {
    pub tool_use_id: String,
    pub content: String,
    pub is_error: bool,
}

pub struct StandupAgent {
    client: reqwest::Client,
    app_state: AppState,
}

impl StandupAgent {
    pub fn new(app_state: AppState) -> Self {
        let client = reqwest::Client::new();
        Self { client, app_state }
    }

    #[allow(clippy::too_many_lines)]
    pub async fn generate_standup_message(&self) -> cja::Result<()> {
        // Create a new thread for this standup generation
        let thread = Thread::create(
            &self.app_state.db,
            "Generate daily standup message".to_string(),
        )
        .await?;

        // Update thread status to running
        Thread::update_status(&self.app_state.db, thread.thread_id, "running").await?;

        let thread_id = thread.thread_id;
        let mut previous_stitch_id: Option<Uuid> = None;

        // Run the standup loop and handle completion
        let result = self
            .run_standup_loop(thread_id, &mut previous_stitch_id)
            .await;

        // Handle thread completion based on result
        match result {
            Ok(()) => {
                Thread::complete(
                    &self.app_state.db,
                    thread_id,
                    json!({
                        "success": true,
                        "message": "Standup message sent successfully"
                    }),
                )
                .await?;
            }
            Err(StandupLoopError::AbortedLoop) => {
                Thread::abort(
                    &self.app_state.db,
                    thread_id,
                    json!({
                        "success": false,
                        "error": "Thread aborted: Maximum message limit reached"
                    }),
                )
                .await?;

                return Err(cja::color_eyre::eyre::eyre!(
                    "Thread aborted: Maximum message limit reached"
                ));
            }
            Err(StandupLoopError::Other(e)) => {
                Thread::fail(
                    &self.app_state.db,
                    thread_id,
                    json!({
                        "success": false,
                        "error": e.to_string()
                    }),
                )
                .await?;

                return Err(e);
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    async fn run_standup_loop(
        &self,
        thread_id: Uuid,
        previous_stitch_id: &mut Option<Uuid>,
    ) -> Result<(), StandupLoopError> {
        let channel_id = self.app_state.standup.discord_channel_id.ok_or_else(|| {
            cja::color_eyre::eyre::eyre!("DAILY_MESSAGE_DISCORD_CHANNEL_ID not configured")
        })?;
        let user_id = self.app_state.standup.discord_user_id.ok_or_else(|| {
            cja::color_eyre::eyre::eyre!("DAILY_MESSAGE_DISCORD_USER_ID not configured")
        })?;

        let continue_looping = Arc::new(Mutex::new(true));

        let mut tools = ToolBag::default();
        tools.add_tool(SendDiscordMessage::new(self.app_state.clone()))?;
        tools.add_tool(DoneTool::new(continue_looping.clone()))?;
        tools.add_tool(
            crate::al::tools::tool_suggestions::ToolSuggestionsSubmit::new(self.app_state.clone()),
        )?;

        let now_eastern = Utc::now().with_timezone(&Eastern);
        let date_str = now_eastern.format("%A, %B %d, %Y at %I:%M %p").to_string();

        let prompt = format!(
            "You are a friendly AI assistant helping with daily standup messages. \
            Generate a warm, encouraging good morning message for a developer's daily standup. \
            Keep it brief (2-3 sentences max), professional but friendly. \
            Include the current time: {date_str}. \
            Vary the message each day - be creative but appropriate for a work context. \
            End with encouragement for the standup meeting.
            Send the message to the following discord channel: {channel_id}
            And tag the following user: {user_id}"
        );

        let mut messages = vec![Message {
            role: "user".to_string(),
            content: vec![Content::Text(TextContent { text: prompt })],
        }];

        const MAX_MESSAGES: usize = 100;

        while *continue_looping.lock().await {
            if messages.len() > MAX_MESSAGES {
                return Err(StandupLoopError::AbortedLoop);
            }

            let request = AnthropicRequest {
                model: "claude-sonnet-4-0".to_string(),
                max_tokens: 1024,
                messages: messages.clone(),
                tools: tools.as_api(),
                tool_choice: Some(ToolChoice {
                    r#type: "any".to_string(),
                }),
            };

            let response = self
                .client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", &self.app_state.standup.anthropic_api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&request)
                .send()
                .await?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                return Err(StandupLoopError::Other(cja::color_eyre::eyre::eyre!(
                    "Anthropic API error: {}",
                    error_text
                )));
            }

            let response_data: AnthropicResponse = response.json().await?;

            let llm_stitch = Stitch::create_llm_call(
                &self.app_state.db,
                thread_id,
                *previous_stitch_id,
                serde_json::to_value(&request)?,
                serde_json::to_value(&response_data)?,
            )
            .await?;

            *previous_stitch_id = Some(llm_stitch.stitch_id);

            messages.push(Message {
                role: "assistant".to_string(),
                content: response_data.content.clone(),
            });

            let mut tool_results = vec![];

            for content in response_data.content {
                match content {
                    Content::Text(_text) => {
                        // Text content from assistant - no action needed
                    }
                    Content::ToolUse(tool_use_content) => {
                        *continue_looping.lock().await = true;

                        let tool_use_id = tool_use_content.id.clone();
                        let tool_name = tool_use_content.name.clone();
                        let tool_input = tool_use_content.input.clone();

                        // Create thread context with the previous stitch ID
                        let context = ThreadContext {
                            thread_id,
                            previous_stitch_id: *previous_stitch_id,
                        };

                        let tool_result = match tools.call_tool(tool_use_content, context).await {
                            Ok(tool_result) => {
                                // Create tool call stitch for successful execution
                                let tool_stitch = Stitch::create_tool_call(
                                    &self.app_state.db,
                                    thread_id,
                                    *previous_stitch_id,
                                    tool_name,
                                    tool_input,
                                    serde_json::to_value(&tool_result)?,
                                )
                                .await?;

                                *previous_stitch_id = Some(tool_stitch.stitch_id);

                                ToolResult {
                                    tool_use_id,
                                    content: serde_json::to_string(&tool_result)?,
                                    is_error: false,
                                }
                            }
                            Err(e) => {
                                // Create tool call stitch for error
                                let tool_stitch = Stitch::create_tool_call(
                                    &self.app_state.db,
                                    thread_id,
                                    *previous_stitch_id,
                                    tool_name,
                                    tool_input,
                                    json!({"error": e.to_string()}),
                                )
                                .await?;

                                *previous_stitch_id = Some(tool_stitch.stitch_id);

                                ToolResult {
                                    tool_use_id,
                                    content: format!("Tool error: {e}"),
                                    is_error: true,
                                }
                            }
                        };
                        tool_results.push(Content::ToolResult(tool_result));
                    }
                    Content::ToolResult(_) => {
                        unreachable!("ToolResult should not appear in assistant response")
                    }
                }
            }

            messages.push(Message {
                role: "user".to_string(),
                content: tool_results,
            });
        }

        Ok(())
    }
}
