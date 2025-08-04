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
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    http_server::{ResponseResult, ServerError},
    AppState,
};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinearWebhookPayload {
    action: String,
    #[serde(rename = "type")]
    event_type: String,
    data: Value,
    url: Option<String>,
    #[serde(rename = "createdAt")]
    created_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct LinearAgentActivity {
    #[serde(rename = "type")]
    activity_type: LinearActivityType,
    message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum LinearActivityType {
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

async fn emit_agent_activity(
    webhook_url: &str,
    activity: LinearAgentActivity,
) -> Result<(), ServerError> {
    let client = reqwest::Client::new();

    let response = client
        .post(webhook_url)
        .json(&activity)
        .send()
        .await
        .wrap_err("Failed to emit agent activity")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(cja::color_eyre::eyre::eyre!(
            "Failed to emit agent activity: {status} - {body}"
        )
        .into());
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

    match (payload.event_type.as_str(), payload.action.as_str()) {
        ("AgentSession", "created") => {
            info!("Agent session created, acknowledging with thought activity");

            if let Some(webhook_url) = payload.url {
                let activity = LinearAgentActivity {
                    activity_type: LinearActivityType::Thought,
                    message: "Processing request...".to_string(),
                };

                if let Err(e) = emit_agent_activity(&webhook_url, activity).await {
                    error!("Failed to emit initial thought activity: {e}");
                }
            }
        }
        _ => {
            info!(
                event_type = payload.event_type,
                action = payload.action,
                "Unhandled webhook event type"
            );
        }
    }

    sqlx::query!(
        r#"
        UPDATE linear_webhook_events
        SET processed_at = NOW()
        WHERE linear_webhook_event_id = $1
        "#,
        event_id
    )
    .execute(&app_state.db)
    .await?;

    ResponseResult::Ok(StatusCode::OK)
}
