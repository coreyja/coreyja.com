use chrono::{Datelike, Utc};
use db::agentic_threads::{Stitch, Thread};
use serde::{Deserialize, Serialize};

use crate::{jobs::thread_processor::ProcessThreadStep, AppState};
use cja::{chrono_tz::US::Eastern, jobs::Job};

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub messages: Vec<Message>,
    pub tools: Vec<AnthropicTool>,
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
}

#[derive(Debug, Serialize)]
pub struct ThinkingConfig {
    pub r#type: String,
    pub budget_tokens: u32,
}

#[derive(Debug, Serialize)]
pub struct ToolChoice {
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: Vec<Content>,
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
    #[serde(rename = "image")]
    Image(ImageContent),
    #[serde(rename = "document")]
    Document(DocumentContent),
    #[serde(rename = "tool_use")]
    ToolUse(ToolUseContent),
    #[serde(rename = "tool_result")]
    ToolResult(ToolResult),
    #[serde(rename = "thinking")]
    Thinking(ThinkingContent),
}

impl Content {
    pub fn set_cache_control(&mut self, cache_control: CacheControl) {
        match self {
            Content::Text(text_content) => {
                text_content.cache_control = Some(cache_control);
            }
            Content::Image(image_content) => {
                image_content.cache_control = Some(cache_control);
            }
            Content::Document(document_content) => {
                document_content.cache_control = Some(cache_control);
            }
            Content::ToolUse(tool_use_content) => {
                tool_use_content.cache_control = Some(cache_control);
            }
            Content::ToolResult(tool_result) => {
                tool_result.cache_control = Some(cache_control);
            }
            Content::Thinking(thinking_content) => {
                thinking_content.cache_control = Some(cache_control);
            }
        }
    }

    pub fn cache_control(&self) -> Option<&CacheControl> {
        match self {
            Content::Text(text_content) => text_content.cache_control.as_ref(),
            Content::Image(image_content) => image_content.cache_control.as_ref(),
            Content::Document(document_content) => document_content.cache_control.as_ref(),
            Content::ToolUse(tool_use_content) => tool_use_content.cache_control.as_ref(),
            Content::ToolResult(tool_result) => tool_result.cache_control.as_ref(),
            Content::Thinking(thinking_content) => thinking_content.cache_control.as_ref(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TextContent {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ImageContent {
    pub source: ImageSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ImageSource {
    pub r#type: String, // "base64" or "url"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>, // "image/jpeg", "image/png", "image/gif", "image/webp"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>, // base64-encoded image data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>, // URL to image
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct DocumentContent {
    pub source: DocumentSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct DocumentSource {
    pub r#type: String, // "base64" or "url"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>, // "application/pdf"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>, // base64-encoded document data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>, // URL to document
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ToolUseContent {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ToolResult {
    pub tool_use_id: String,
    pub content: String,
    pub is_error: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ThinkingContent {
    pub thinking: String,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

pub struct StandupAgent {
    app_state: AppState,
}

impl StandupAgent {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    pub async fn generate_standup_message(&self) -> cja::Result<()> {
        let now_eastern = Utc::now().with_timezone(&Eastern);
        let date_str = now_eastern.format("%A, %B %d, %Y at %I:%M %p").to_string();
        let day_of_week = now_eastern.date_naive().weekday().to_string();
        let thread_name = format!("Standup for {date_str}");

        // Get the agent configuration for standup (use Al agent)
        let agent_id = crate::agent_config::AgentId::Al;
        let agent_config = agent_id.config();

        // Create thread using the agent - agent handles Discord thread creation and setup
        let thread = agent_config
            .create_thread(&self.app_state.discord, &self.app_state.db, thread_name)
            .await?
            .with_goal("Generate daily standup message")
            .build()
            .await?;

        // Update thread status to running
        Thread::update_status(&self.app_state.db, thread.thread_id, "running").await?;

        let user_id = std::env::var("DAILY_MESSAGE_DISCORD_USER_ID").map_err(|_| {
            cja::color_eyre::eyre::eyre!(
                "Environment variable 'DAILY_MESSAGE_DISCORD_USER_ID' not set"
            )
        })?;

        // Get Linear API key from environment or thread metadata
        let api_key = crate::al::tools::linear_graphql::get_linear_api_key(&self.app_state).await?;

        const DEV_TEAM_ID: &str = "affbc39a-f0d9-467c-9d67-d221e570bb19";
        const COREY_USER_ID: &str = "14817796-bf9f-4444-910a-a2ed1db58f23";
        let standup_data =
            crate::linear::graphql::get_standup_data(&api_key, DEV_TEAM_ID, COREY_USER_ID).await?;
        let standup_data_json = serde_json::to_string(&standup_data)?;

        let prompt = format!(
            r"
            You are my daily work planning assistant. Help me review my progress and plan a focused day.

            <date>
            {date_str}
            </date>

            <day_of_week>
            {day_of_week}
            </day_of_week>

            ## Linear Context
            <linear_data>
            {standup_data_json}
            </linear_data>

            ## Your Task
            Analyze the Linear data and have a conversational planning session with me:

            ### Quick Progress Check
            - Look at what I updated/closed in the last 24 hours (or since Friday if today is Monday)
            - Briefly acknowledge the wins and completed work
            - Check how the current cycle is progressing and note if we're on track
            - Ask if I did anything else that wasn't tracked (we should add completed cards to capture all work)

            ### Today's Focus
            - Check my in-progress issues
            - Help me identify THE one thing to focus on today:
              - If I have nothing in progress: Suggest the most important unassigned issue from the current cycle (or backlog if needed)
              - If I have multiple things in progress: Ask which one should be today's priority
            - Consider what's realistically achievable in a single day's deep work
            - Any blockers I should address first before diving in?

            ### Conversation Style
            Be conversational and direct - this is just between us. Use my user id {user_id} naturally when referring to me. Make issue IDs (e.g., DEV-123) clickable links to Linear.

            Don't create a formatted update - just talk through what got done, what today's main focus should be and why. Keep it brief and actionable - this is about planning my day, not reporting to anyone.
            Make sure to tag my in the response or else I won't get a notification for the new thread!
            "
        );
        let prompt = prompt.trim();

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
