use chrono::Utc;
use miette::{Context, IntoDiagnostic};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct GoogleConfig {
    pub client_id: String,
    pub client_secret: String,
}

impl GoogleConfig {
    #[tracing::instrument(name = "GoogleConfig::from_env")]
    pub fn from_env() -> miette::Result<Self> {
        Ok(Self {
            client_id: std::env::var("GOOGLE_CLIENT_ID")
                .into_diagnostic()
                .context("GOOGLE_CLIENT_ID env var missing")?,
            client_secret: std::env::var("GOOGLE_CLIENT_SECRET")
                .into_diagnostic()
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

pub(crate) async fn get_valid_google_token(app_state: &AppState) -> miette::Result<String> {
    let google_user = sqlx::query!(
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
    .await
    .into_diagnostic()?;

    if google_user.access_token_expires_at < Utc::now() {
        // TODO: Refresh token
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
            .await
            .into_diagnostic()?;

        let token_info: GitHubTokenResponse = res.json().await.into_diagnostic()?;

        let encrypted_access_token = app_state.encrypt_config.encrypt(&token_info.access_token)?;

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
            token_info.expires_in as f64,
        )
        .execute(&app_state.db)
        .await
        .into_diagnostic()?;

        return Ok(token_info.access_token);
    }

    let access_token = app_state
        .encrypt_config
        .decrypt(&google_user.encrypted_access_token)?;

    Ok(access_token)
}
