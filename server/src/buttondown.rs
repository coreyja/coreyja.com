use chrono::{DateTime, Utc};
use cja::color_eyre::eyre::Context;
use serde::{Deserialize, Serialize};

const BUTTONDOWN_API_BASE: &str = "https://api.buttondown.com/v1";

#[derive(Debug, Clone)]
pub struct ButtondownConfig {
    pub api_key: String,
}

impl ButtondownConfig {
    pub fn from_env() -> cja::Result<Self> {
        Ok(Self {
            api_key: std::env::var("BUTTONDOWN_API_KEY")
                .context("BUTTONDOWN_API_KEY env var missing")?,
        })
    }
}

/// Status for creating emails via the Buttondown API
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmailStatus {
    /// Send immediately
    AboutToSend,
    /// Schedule for later (requires `publish_date`)
    Scheduled,
    /// Save as draft
    Draft,
}

/// Request body for creating an email
#[derive(Debug, Clone, Serialize)]
pub struct CreateEmailRequest {
    pub subject: String,
    pub body: String,
    pub status: EmailStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish_date: Option<DateTime<Utc>>,
}

/// Response from creating an email
#[derive(Debug, Clone, Deserialize)]
pub struct CreateEmailResponse {
    pub id: String,
}

/// Client for interacting with the Buttondown API
pub struct ButtondownClient {
    client: reqwest::Client,
    api_key: String,
}

impl ButtondownClient {
    pub fn new(config: &ButtondownConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: config.api_key.clone(),
        }
    }

    /// Create a new email in Buttondown
    pub async fn create_email(
        &self,
        request: &CreateEmailRequest,
    ) -> cja::Result<CreateEmailResponse> {
        let url = format!("{BUTTONDOWN_API_BASE}/emails");

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Token {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .context("Failed to send request to Buttondown API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read response body".to_string());
            return Err(cja::color_eyre::eyre::eyre!(
                "Buttondown API returned error {}: {}",
                status,
                body
            ));
        }

        response
            .json::<CreateEmailResponse>()
            .await
            .context("Failed to parse Buttondown API response")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_status_serialization() {
        assert_eq!(
            serde_json::to_string(&EmailStatus::AboutToSend).unwrap(),
            "\"about_to_send\""
        );
        assert_eq!(
            serde_json::to_string(&EmailStatus::Scheduled).unwrap(),
            "\"scheduled\""
        );
        assert_eq!(
            serde_json::to_string(&EmailStatus::Draft).unwrap(),
            "\"draft\""
        );
    }

    #[test]
    fn test_create_email_request_serialization() {
        let request = CreateEmailRequest {
            subject: "Test Newsletter".to_string(),
            body: "Hello, world!".to_string(),
            status: EmailStatus::AboutToSend,
            publish_date: None,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["subject"], "Test Newsletter");
        assert_eq!(json["body"], "Hello, world!");
        assert_eq!(json["status"], "about_to_send");
        assert!(json.get("publish_date").is_none());
    }

    #[test]
    fn test_create_email_request_with_schedule() {
        let publish_date = DateTime::parse_from_rfc3339("2026-01-25T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let request = CreateEmailRequest {
            subject: "Scheduled Newsletter".to_string(),
            body: "Hello, future!".to_string(),
            status: EmailStatus::Scheduled,
            publish_date: Some(publish_date),
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["status"], "scheduled");
        assert!(json.get("publish_date").is_some());
    }
}
