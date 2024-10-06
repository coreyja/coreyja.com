use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use cja::{app_state::AppState as _, color_eyre::eyre::Context, jobs::Job as _};
use serde_json::{json, Value};

use crate::{
    http_server::{errors::WithStatus as _, ResponseResult},
    jobs::bytes_discord_posts::PostByteSubmission,
    AppState,
};

#[derive(serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Payload {
    subdomain: String,
    slug: String,
    player_response: Option<String>,
    score: i32,
}

#[axum_macros::debug_handler]
pub(crate) async fn handler(
    State(state): State<AppState>,
    Json(webhook_payload): Json<Value>,
) -> ResponseResult<impl IntoResponse> {
    let payload: Payload = serde_json::from_value(webhook_payload)
        .context("Could not parse payload into expected JSON")
        .with_status(StatusCode::UNPROCESSABLE_ENTITY)?;

    let now = chrono::Utc::now();

    let db_result = sqlx::query!(
        r#"
        INSERT INTO CookdWebhooks (subdomain, slug, player_github_username, score, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6) RETURNING cookd_webhook_id
        "#,
        payload.subdomain,
        payload.slug,
        payload.player_response,
        payload.score,
        now,
        now
    ).fetch_one(state.db()).await.context("Could not insert webhook payload into database")?;

    let job = PostByteSubmission {
        cookd_webhook_id: db_result.cookd_webhook_id,
    };

    job.enqueue(state.clone(), "Cookd Webhook".to_string())
        .await
        .context("Could not enqueue job")?;

    Ok(Json(
        json!({ "cookd_webhook_id": db_result.cookd_webhook_id }),
    ))
}
