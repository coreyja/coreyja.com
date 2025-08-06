use axum::{
    extract::{rejection::BytesRejection, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use cja::color_eyre::eyre::Context;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    http_server::{ResponseResult, ServerError},
    jobs::linear_webhook_processor::ProcessLinearWebhook,
    AppState,
};
use cja::jobs::Job as JobTrait;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinearWebhookPayload {
    pub action: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: Value,
    pub url: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinearAgentActivity {
    #[serde(rename = "type")]
    pub activity_type: LinearActivityType,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LinearActivityType {
    Thought,
    Action,
    Response,
    Error,
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

    let payload: LinearWebhookPayload =
        serde_json::from_slice(&body).wrap_err("Failed to parse webhook payload")?;

    info!(
        action = payload.action,
        event_type = payload.event_type,
        "Received Linear webhook"
    );

    let event_id = sqlx::query!(
        r#"
        INSERT INTO linear_webhook_events (linear_webhook_event_id, event_type, payload)
        VALUES ($1, $2, $3)
        RETURNING linear_webhook_event_id
        "#,
        Uuid::new_v4(),
        format!("{}.{}", payload.event_type, payload.action),
        serde_json::to_value(&payload)?
    )
    .fetch_one(&app_state.db)
    .await?
    .linear_webhook_event_id;

    // Enqueue the job to process the webhook in the background
    ProcessLinearWebhook { payload }
        .enqueue(app_state.clone(), "Linear webhook processing".to_string())
        .await?;

    info!(
        event_id = %event_id,
        "Linear webhook enqueued for background processing"
    );

    ResponseResult::Ok(StatusCode::OK)
}
