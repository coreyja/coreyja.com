use axum::{extract::State, http::StatusCode, Json};
use cja::{app_state::AppState as _, color_eyre::eyre::Context};
use serde_json::{json, Value};

use crate::{
    http_server::{errors::WithStatus as _, ResponseResult},
    AppState,
};

#[derive(serde::Deserialize, Clone)]
struct Payload {
    subdomain: String,
    slug: String,
    player_github_email: Option<String>,
    player_github_username: Option<String>,
    score: i32,
}

pub(crate) async fn handler(
    State(state): State<AppState>,
    Json(webhook_payload): Json<Value>,
) -> ResponseResult<Value> {
    let payload: Payload = serde_json::from_value(webhook_payload)
        .context("Could not parse payload into expected JSON")
        .with_status(StatusCode::UNPROCESSABLE_ENTITY)?;

    let db_result = sqlx::query!(
        r#"
        INSERT INTO CookdWebhooks (subdomain, slug, player_github_email, player_github_username, score)
        VALUES ($1, $2, $3, $4, $5) RETURNING cookd_webhook_id
        "#,
        payload.subdomain,
        payload.slug,
        payload.player_github_email,
        payload.player_github_username,
        payload.score,
    ).fetch_one(state.db()).await.context("Could not insert webhook payload into database")?;

    Ok(json!({ "cookd_webhook_id": db_result.cookd_webhook_id }))
}
