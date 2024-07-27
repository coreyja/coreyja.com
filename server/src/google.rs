use chrono::Utc;
use cja::color_eyre::eyre::Context;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct GoogleConfig {
    pub client_id: String,
    pub client_secret: String,
}

impl GoogleConfig {
    #[tracing::instrument(name = "GoogleConfig::from_env")]
    pub fn from_env() -> cja::Result<Self> {
        Ok(Self {
            client_id: std::env::var("GOOGLE_CLIENT_ID")
                .context("GOOGLE_CLIENT_ID env var missing")?,
            client_secret: std::env::var("GOOGLE_CLIENT_SECRET")
                .context("GOOGLE_CLIENT_SECRET env var missing")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct GitHubTokenResponse {
    pub(crate) access_token: String,
    pub(crate) expires_in: u64,
    pub(crate) scope: String,
    pub(crate) token_type: String,
}

#[tracing::instrument(
    name = "refresh_google_token",
    skip_all,
    fields(google_user_id = %google_user.google_user_id)
)]
pub(crate) async fn refresh_google_token(
    app_state: &AppState,
    google_user: &GoogleUser,
) -> cja::Result<String> {
    let refresh_token = app_state
        .encrypt_config
        .decrypt(&google_user.encrypted_refresh_token)?;

    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "refresh_token"),
        ("client_id", &app_state.google.client_id),
        ("client_secret", &app_state.google.client_secret),
        ("refresh_token", &refresh_token),
    ];

    let res = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await?;

    let token_info: GitHubTokenResponse = res.json().await?;

    let encrypted_access_token = app_state.encrypt_config.encrypt(&token_info.access_token)?;

    let expires_in: u32 = token_info
        .expires_in
        .try_into()
        .context("expires_in did not fit in a u32")?;
    let expires_in: f64 = expires_in.into();
    sqlx::query!(
        "
    UPDATE GoogleUsers
    SET
        encrypted_access_token = $1,
        access_token_expires_at = NOW() + (INTERVAL '1 second' * $3)
    WHERE google_user_id = $2
    ",
        encrypted_access_token,
        google_user.google_user_id,
        expires_in,
    )
    .execute(&app_state.db)
    .await?;

    Ok(token_info.access_token)
}

pub(crate) struct GoogleUser {
    #[allow(clippy::struct_field_names)]
    google_user_id: Uuid,
    encrypted_access_token: Vec<u8>,
    encrypted_refresh_token: Vec<u8>,
    access_token_expires_at: chrono::DateTime<chrono::Utc>,
}

pub(crate) async fn get_valid_google_token(app_state: &AppState) -> cja::Result<String> {
    let google_user = sqlx::query_as!(
        GoogleUser,
        "
        SELECT
            google_user_id,
            encrypted_access_token,
            encrypted_refresh_token,
            access_token_expires_at
        FROM GoogleUsers
        LIMIT 1
        "
    )
    .fetch_one(&app_state.db)
    .await?;

    if google_user.access_token_expires_at < Utc::now() {
        return refresh_google_token(app_state, &google_user).await;
    }

    let access_token = app_state
        .encrypt_config
        .decrypt(&google_user.encrypted_access_token)?;

    Ok(access_token)
}
