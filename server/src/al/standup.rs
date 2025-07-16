use chrono::Utc;
use chrono_tz::US::Eastern;
use serde::{Deserialize, Serialize};

use crate::{
    al::tools::{discord::SendDiscordMessage, ToolBag},
    AppState,
};

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
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<Content>,
}

#[derive(Debug, Deserialize)]
struct Content {
    text: String,
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

    pub async fn generate_standup_message(&self) -> cja::Result<String> {
        let channel_id = self.app_state.standup.discord_channel_id.ok_or_else(|| {
            cja::color_eyre::eyre::eyre!("DAILY_MESSAGE_DISCORD_CHANNEL_ID not configured")
        })?;
        let user_id = self.app_state.standup.discord_user_id.ok_or_else(|| {
            cja::color_eyre::eyre::eyre!("DAILY_MESSAGE_DISCORD_USER_ID not configured")
        })?;

        let mut tools = ToolBag::default();
        tools.add_tool(SendDiscordMessage::new(self.app_state.clone()))?;

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

        let request = AnthropicRequest {
            model: "claude-3-5-haiku-latest".to_string(),
            max_tokens: 150,
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            tools: tools.as_api(),
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
            return Err(cja::color_eyre::eyre::eyre!(
                "Anthropic API error: {}",
                error_text
            ));
        }

        let response_data: AnthropicResponse = response.json().await?;
        let message = response_data
            .content
            .first()
            .ok_or_else(|| cja::color_eyre::eyre::eyre!("No content in Anthropic response"))?
            .text
            .clone();

        Ok(message)
    }
}
