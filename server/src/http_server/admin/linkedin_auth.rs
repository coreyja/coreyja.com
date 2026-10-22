use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{http_server::auth::session::AdminUser, state::AppState};

const LINKEDIN_AUTH_URL: &str = "https://www.linkedin.com/oauth/v2/authorization";
const LINKEDIN_TOKEN_URL: &str = "https://www.linkedin.com/oauth/v2/accessToken";
const LINKEDIN_USERINFO_URL: &str = "https://api.linkedin.com/v2/userinfo";

/// Personal-profile posting scopes:
/// - `openid` + `profile` give us the `sub` claim via /v2/userinfo
/// - `w_member_social` is the narrowest write scope to post as the user
const LINKEDIN_SCOPE: &str = "openid profile w_member_social";

/// CSRF state lifetime. After this many minutes the callback rejects the row
/// rather than honoring it. Keeps the security guarantee comparable to a
/// short-lived signed cookie.
const OAUTH_STATE_TTL_MINUTES: i64 = 10;

pub(crate) async fn linkedin_auth(
    State(app_state): State<AppState>,
    _: AdminUser,
) -> Result<impl IntoResponse, String> {
    let Some(linkedin) = app_state.linkedin.as_ref() else {
        return Err(
            "LinkedIn not configured. Set LINKEDIN_CLIENT_ID and LINKEDIN_CLIENT_SECRET env vars and restart."
                .to_string(),
        );
    };

    // DB-backed CSRF state — same pattern as LinearOauthStates. Each redirect
    // inserts a fresh row; the callback validates the `state` query param
    // against it and enforces a 10-minute freshness window so a stolen/leaked
    // state value can't be replayed indefinitely.
    let state_id = Uuid::new_v4();
    let state_value = Uuid::new_v4().to_string();

    sqlx::query!(
        r#"
        INSERT INTO LinkedInOauthStates (linkedin_oauth_state_id, state)
        VALUES ($1, $2)
        "#,
        state_id,
        state_value,
    )
    .execute(&app_state.db)
    .await
    .map_err(|e| format!("Failed to record OAuth state: {e}"))?;

    let redirect_uri = app_state.app.app_url("/admin/auth/linkedin/callback");
    let auth_url = url::Url::parse_with_params(
        LINKEDIN_AUTH_URL,
        &[
            ("response_type", "code"),
            ("client_id", &linkedin.client_id),
            ("redirect_uri", &redirect_uri),
            ("scope", LINKEDIN_SCOPE),
            ("state", &state_value),
        ],
    )
    .map_err(|e| format!("Failed to build LinkedIn auth URL: {e}"))?
    .to_string();

    Ok(Redirect::to(&auth_url))
}

#[derive(Debug, Deserialize)]
struct LinkedInTokenResponse {
    access_token: String,
    expires_in: i64,
    refresh_token: String,
    refresh_token_expires_in: i64,
    scope: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct LinkedInUserInfo {
    sub: String,
}

#[allow(clippy::too_many_lines)]
pub(crate) async fn linkedin_auth_callback(
    State(app_state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    admin: AdminUser,
) -> Result<impl IntoResponse, String> {
    let Some(linkedin) = app_state.linkedin.as_ref() else {
        return Err("LinkedIn not configured.".to_string());
    };

    let code = params
        .get("code")
        .ok_or_else(|| "No code in callback query".to_string())?;
    let state = params
        .get("state")
        .ok_or_else(|| "No state in callback query".to_string())?;

    // Validate state against the DB, requiring the row to be no older than
    // OAUTH_STATE_TTL_MINUTES. On success we delete the row so it can't be
    // replayed. Also opportunistically sweep any other stale rows so the
    // table doesn't grow unboundedly from abandoned "Connect LinkedIn" clicks.
    let ttl_cutoff = chrono::Utc::now() - chrono::Duration::minutes(OAUTH_STATE_TTL_MINUTES);
    let matched = sqlx::query!(
        "SELECT linkedin_oauth_state_id FROM LinkedInOauthStates \
         WHERE state = $1 AND created_at >= $2",
        state,
        ttl_cutoff,
    )
    .fetch_optional(&app_state.db)
    .await
    .map_err(|e| format!("Failed to look up OAuth state: {e}"))?;

    let Some(matched) = matched else {
        return Err("Invalid or expired OAuth state".to_string());
    };

    sqlx::query!(
        "DELETE FROM LinkedInOauthStates \
         WHERE linkedin_oauth_state_id = $1 OR created_at < $2",
        matched.linkedin_oauth_state_id,
        ttl_cutoff,
    )
    .execute(&app_state.db)
    .await
    .map_err(|e| format!("Failed to clear OAuth state: {e}"))?;

    let redirect_uri = app_state.app.app_url("/admin/auth/linkedin/callback");
    let http = reqwest::Client::new();
    let token_resp = http
        .post(LINKEDIN_TOKEN_URL)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &redirect_uri),
            ("client_id", &linkedin.client_id),
            ("client_secret", &linkedin.client_secret),
        ])
        .send()
        .await
        .map_err(|e| format!("Token exchange request failed: {e}"))?;

    if !token_resp.status().is_success() {
        let status = token_resp.status();
        let body = token_resp.text().await.unwrap_or_default();
        return Err(format!("LinkedIn token exchange failed ({status}): {body}"));
    }

    let token_data: LinkedInTokenResponse = token_resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {e}"))?;

    let userinfo_resp = http
        .get(LINKEDIN_USERINFO_URL)
        .bearer_auth(&token_data.access_token)
        .send()
        .await
        .map_err(|e| format!("Userinfo request failed: {e}"))?;

    if !userinfo_resp.status().is_success() {
        let status = userinfo_resp.status();
        let body = userinfo_resp.text().await.unwrap_or_default();
        return Err(format!("LinkedIn userinfo failed ({status}): {body}"));
    }

    let user_info: LinkedInUserInfo = userinfo_resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse userinfo: {e}"))?;

    let encrypted_access = app_state
        .encrypt_config
        .encrypt(&token_data.access_token)
        .map_err(|e| format!("Failed to encrypt access token: {e}"))?;
    let encrypted_refresh = app_state
        .encrypt_config
        .encrypt(&token_data.refresh_token)
        .map_err(|e| format!("Failed to encrypt refresh token: {e}"))?;

    let access_expires_at = chrono::Utc::now() + chrono::Duration::seconds(token_data.expires_in);
    let refresh_expires_at =
        chrono::Utc::now() + chrono::Duration::seconds(token_data.refresh_token_expires_in);

    sqlx::query!(
        r#"
        INSERT INTO LinkedInUsers
          (linkedin_user_id, user_id, external_linkedin_id,
           encrypted_access_token, access_token_expires_at,
           encrypted_refresh_token, refresh_token_expires_at, scope)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (user_id) DO UPDATE SET
          external_linkedin_id = EXCLUDED.external_linkedin_id,
          encrypted_access_token = EXCLUDED.encrypted_access_token,
          access_token_expires_at = EXCLUDED.access_token_expires_at,
          encrypted_refresh_token = EXCLUDED.encrypted_refresh_token,
          refresh_token_expires_at = EXCLUDED.refresh_token_expires_at,
          scope = EXCLUDED.scope,
          updated_at = NOW()
        "#,
        Uuid::new_v4(),
        admin.user_id,
        user_info.sub,
        encrypted_access,
        access_expires_at,
        encrypted_refresh,
        refresh_expires_at,
        token_data.scope,
    )
    .execute(&app_state.db)
    .await
    .map_err(|e| format!("Failed to persist LinkedIn user: {e}"))?;

    Ok(Redirect::to("/admin"))
}
