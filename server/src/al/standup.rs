use chrono::Utc;
use chrono_tz::US::Eastern;
use rig::agent::Agent;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::anthropic;

pub struct StandupAgent {
    agent: Agent<anthropic::completion::CompletionModel>,
}

impl StandupAgent {
    pub fn new(client: &anthropic::Client) -> Self {
        let agent = client.agent("claude-3-5-haiku-latest").build();
        Self { agent }
    }

    pub async fn generate_standup_message(&self) -> cja::Result<String> {
        let now_eastern = Utc::now().with_timezone(&Eastern);
        let date_str = now_eastern.format("%A, %B %d, %Y at %I:%M %p").to_string();

        let prompt = format!(
            "You are a friendly AI assistant helping with daily standup messages. \
            Generate a warm, encouraging good morning message for a developer's daily standup. \
            Keep it brief (2-3 sentences max), professional but friendly. \
            Include the current time: {date_str}. \
            Vary the message each day - be creative but appropriate for a work context. \
            End with encouragement for the standup meeting."
        );

        let response = self.agent.prompt(&prompt).await?;

        Ok(response)
    }
}
