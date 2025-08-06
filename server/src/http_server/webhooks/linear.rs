use axum::{
    extract::{rejection::BytesRejection, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use cja::color_eyre::eyre::Context;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tracing::{info, warn};

use crate::{
    http_server::{ResponseResult, ServerError},
    jobs::linear_webhook_processor::ProcessLinearWebhook,
    AppState,
};
use cja::jobs::Job as JobTrait;

type HmacSha256 = Hmac<Sha256>;

// Generic webhook payload structure
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum LinearWebhookPayload {
    AgentSessionEvent(AgentSessionEventPayload),
}

// Agent session event specific payload
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentSessionEventPayload {
    pub action: String, // "created" or "prompted"
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "organizationId")]
    pub organization_id: String,
    #[serde(rename = "oauthClientId")]
    pub oauth_client_id: String,
    #[serde(rename = "agentSession")]
    pub agent_session: AgentSession,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentSession {
    pub id: String,
    pub issue: Option<LinearIssueContext>,
    pub comment: Option<LinearCommentContext>,
    #[serde(rename = "previousComments")]
    pub previous_comments: Option<Vec<LinearCommentContext>>,
    #[serde(rename = "agentActivity")]
    pub agent_activity: Option<AgentActivityPrompt>, // Present when action is "prompted"
}

// Minimal typing for issue context - add more fields as needed
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinearIssueContext {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    #[serde(flatten)]
    pub other_fields: serde_json::Map<String, serde_json::Value>,
}

// Minimal typing for comment context - add more fields as needed
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinearCommentContext {
    pub id: String,
    pub body: String,
    #[serde(flatten)]
    pub other_fields: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentActivityPrompt {
    pub body: String,
}

// Fallback for other webhook types we haven't implemented yet
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GenericWebhookPayload {
    pub action: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: serde_json::Value,
    pub url: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

async fn verify_webhook_signature(
    headers: &HeaderMap,
    body: &[u8],
    webhook_secret: &str,
) -> Result<(), ServerError> {
    let signature = headers
        .get("linear-signature")
        .ok_or_else(|| cja::color_eyre::eyre::eyre!("Missing Linear-Signature header"))?
        .to_str()
        .wrap_err("Invalid Linear-Signature header")?;

    let mut mac =
        HmacSha256::new_from_slice(webhook_secret.as_bytes()).wrap_err("Invalid webhook secret")?;
    mac.update(body);

    let expected_signature = hex::encode(mac.finalize().into_bytes());

    if signature != expected_signature {
        return Err(cja::color_eyre::eyre::eyre!("Invalid webhook signature").into());
    }

    Ok(())
}

#[axum_macros::debug_handler(state = AppState)]
pub(crate) async fn linear_webhook(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    body: Result<axum::body::Bytes, BytesRejection>,
) -> impl IntoResponse {
    let body = body?;

    if let Err(e) =
        verify_webhook_signature(&headers, &body, &app_state.linear.webhook_secret).await
    {
        warn!("Webhook signature verification failed: {e}");
        return Err(cja::color_eyre::eyre::eyre!("Unauthorized: Invalid webhook signature").into());
    }

    let generic_payload: serde_json::Value = serde_json::from_slice(&body)?;

    // Log the webhook based on its type
    let (event_type, action) = (
        generic_payload
            .get("type")
            .map(|v| v.as_str().unwrap_or_default())
            .unwrap_or_default()
            .to_string(),
        generic_payload
            .get("action")
            .map(|v| v.as_str().unwrap_or_default())
            .unwrap_or_default()
            .to_string(),
    );

    if let Ok(payload) = serde_json::from_slice(&body) {
        // Enqueue the job to process the webhook in the background
        ProcessLinearWebhook { payload }
            .enqueue(app_state.clone(), "Linear webhook processing".to_string())
            .await?;
    }

    info!(
        event_type,
        action, "Linear webhook enqueued for background processing"
    );

    ResponseResult::Ok(StatusCode::OK)
}
