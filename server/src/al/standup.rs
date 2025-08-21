use chrono::Utc;
use chrono_tz::US::Eastern;
use db::agentic_threads::{Stitch, Thread};
use serde::{Deserialize, Serialize};
use serenity::all::Channel;

use crate::{
    agentic_threads::{builder::DiscordMetadata, ThreadBuilder},
    jobs::thread_processor::ProcessThreadStep,
    AppState,
};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
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

impl Content {
    pub fn set_cache_control(&mut self, cache_control: CacheControl) {
        match self {
            Content::Text(text_content) => {
                text_content.cache_control = Some(cache_control);
            }
            Content::ToolUse(tool_use_content) => {
                tool_use_content.cache_control = Some(cache_control);
            }
            Content::ToolResult(tool_result) => {
                tool_result.cache_control = Some(cache_control);
            }
        }
    }

    pub fn cache_control(&self) -> Option<&CacheControl> {
        match self {
            Content::Text(text_content) => text_content.cache_control.as_ref(),
            Content::ToolUse(tool_use_content) => tool_use_content.cache_control.as_ref(),
            Content::ToolResult(tool_result) => tool_result.cache_control.as_ref(),
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
        let thread_name = format!("Standup for {date_str}");

        // Get configuration for the prompt
        let channel_id = self.app_state.standup.discord_channel_id.ok_or_else(|| {
            cja::color_eyre::eyre::eyre!("DAILY_MESSAGE_DISCORD_CHANNEL_ID not configured")
        })?;
        let user_id = self.app_state.standup.discord_user_id.ok_or_else(|| {
            cja::color_eyre::eyre::eyre!("DAILY_MESSAGE_DISCORD_USER_ID not configured")
        })?;

        let channel_id = serenity::all::ChannelId::new(channel_id);
        let channel = channel_id.to_channel(&self.app_state.discord).await?;
        let Channel::Guild(guild_channel) = channel else {
            color_eyre::eyre::bail!("Channel is not a guild channel");
        };
        let builder = serenity::all::CreateThread::new(thread_name.clone())
            .auto_archive_duration(serenity::all::AutoArchiveDuration::OneDay);
        let new_discord_thread = guild_channel
            .create_thread(&self.app_state.discord, builder)
            .await?;
        let discord_metadata = DiscordMetadata {
            discord_thread_id: new_discord_thread.id.to_string(),
            channel_id: guild_channel.id.to_string(),
            guild_id: guild_channel.guild_id.to_string(),
            created_by: "Al".to_string(),
            thread_name: thread_name.clone(),
        };

        let thread = ThreadBuilder::new(self.app_state.db.clone())
            .with_goal("Generate daily standup message")
            .interactive_discord(discord_metadata)
            .build()
            .await?;

        // Update thread status to running
        Thread::update_status(&self.app_state.db, thread.thread_id, "running").await?;

        // Get Linear API key from environment or thread metadata
        let api_key = crate::al::tools::linear_graphql::get_linear_api_key(&self.app_state).await?;

        const DEV_TEAM_ID: &str = "affbc39a-f0d9-467c-9d67-d221e570bb19";
        const COREY_USER_ID: &str = "14817796-bf9f-4444-910a-a2ed1db58f23";
        let standup_data =
            crate::linear::graphql::get_standup_data(&api_key, DEV_TEAM_ID, COREY_USER_ID).await?;
        let standup_data_json = serde_json::to_string(&standup_data)?;

        let prompt = format!(
            r"
            You are my daily standup assistant. Help me prepare a focused update and plan my day.

            ## Linear Context
            <linear_data>
            {standup_data_json}
            </linear_data>

            ## Your Task
            Analyze the Linear data and help me create a standup update following these steps:

            ### 1. Review Yesterday
            - Identify issues I updated/closed in the last 24 hours (or since Friday if today is Monday)
            - Focus on outcomes achieved, not just activity
            - Ask if there is anything else I completed that wasn't tracked.
                If there is we will want to add completed cards to the Cycle to track how much we are doing even if its not all planned

            ### 2. Plan Today
            - Check my in-progress issues
            - Ensure I have ideally 1 focused tasks:
              - If I have 0: Suggest high-priority unassigned issues from the current cycle. Or search the backlog if this cycle is empty
              - If I have 2+: Ask which to prioritize today
            - Each task should be completable or show meaningful progress today

            ### 4. Output Format
            Create a conversational update (not just bullets) and tag me in the update, my user id is {user_id}:
            - **Yesterday:** What I shipped/completed
            - **Today:** My 1-2 focus areas with clear deliverables

            Make all issue IDs (e.g., DEV-123) clickable links to Linear. Ask clarifying questions about priorities or next tasks. Keep it brief but conversational.
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
