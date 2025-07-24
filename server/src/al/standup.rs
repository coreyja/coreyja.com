use chrono::Utc;
use chrono_tz::US::Eastern;
use db::agentic_threads::{Stitch, Thread};
use serde::{Deserialize, Serialize};

use crate::{jobs::thread_processor::ProcessThreadStep, AppState};
use cja::jobs::Job;

#[derive(Debug, Serialize)]
pub struct AnthropicTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct AnthropicRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<Message>,
    pub tools: Vec<AnthropicTool>,
    pub tool_choice: Option<ToolChoice>,
}

#[derive(Debug, Serialize)]
pub struct ToolChoice {
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheControl {
    pub r#type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AnthropicResponse {
    pub content: Vec<Content>,
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
    app_state: AppState,
}

impl StandupAgent {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    pub async fn generate_standup_message(&self) -> cja::Result<()> {
        // Create a new thread with a high-level goal
        let thread = Thread::create(
            &self.app_state.db,
            "Generate daily standup message".to_string(),
        )
        .await?;

        // Update thread status to running
        Thread::update_status(&self.app_state.db, thread.thread_id, "running").await?;

        // Get configuration for the prompt
        let channel_id = self.app_state.standup.discord_channel_id.ok_or_else(|| {
            cja::color_eyre::eyre::eyre!("DAILY_MESSAGE_DISCORD_CHANNEL_ID not configured")
        })?;
        let user_id = self.app_state.standup.discord_user_id.ok_or_else(|| {
            cja::color_eyre::eyre::eyre!("DAILY_MESSAGE_DISCORD_USER_ID not configured")
        })?;

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

        // Create the initial user message stitch with the full prompt
        Stitch::create_initial_user_message(&self.app_state.db, thread.thread_id, prompt).await?;

        // Create and enqueue the first job to process the thread
        let job = ProcessThreadStep {
            thread_id: thread.thread_id,
        };

        // Enqueue the job to start processing
        job.enqueue(
            self.app_state.clone(),
            "Start standup message generation".to_string(),
        )
        .await?;

        tracing::info!(
            "Enqueued standup generation job for thread {}",
            thread.thread_id
        );

        Ok(())
    }
}
