use cja::jobs::Job;
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshLinkedInToken;

#[async_trait::async_trait]
impl Job<AppState> for RefreshLinkedInToken {
    const NAME: &'static str = "RefreshLinkedInToken";

    async fn run(&self, app_state: AppState) -> cja::Result<()> {
        // LinkedIn token expiry is expected operational state, not exceptional.
        // Propagating Err crashes the server via cja::cron Worker; log + Ok.
        if let Err(e) = do_refresh(&app_state).await {
            tracing::error!("LinkedIn token refresh failed: {:#}", e);
        }
        Ok(())
    }
}

async fn do_refresh(app_state: &AppState) -> cja::Result<()> {
    let Some(linkedin_config) = app_state.linkedin.as_ref() else {
        tracing::info!("LinkedIn not configured (no LINKEDIN_CLIENT_ID/SECRET); skipping refresh");
        return Ok(());
    };

    let row = sqlx::query_as!(
        crate::linkedin::LinkedInUserRow,
        r#"
        SELECT
            linkedin_user_id,
            encrypted_access_token,
            encrypted_refresh_token,
            access_token_expires_at,
            refresh_token_expires_at,
            external_linkedin_id
        FROM LinkedInUsers
        LIMIT 1
        "#
    )
    .fetch_optional(&app_state.db)
    .await?;

    let Some(row) = row else {
        tracing::info!("No LinkedInUsers row yet; admin must complete /admin/auth/linkedin");
        return Ok(());
    };

    if row.refresh_token_expires_at < chrono::Utc::now() + chrono::Duration::days(30) {
        tracing::warn!(
            expires_at = %row.refresh_token_expires_at,
            "LinkedIn refresh token expires soon — re-authorize via /admin/auth/linkedin"
        );
    }

    if row.access_token_expires_at < chrono::Utc::now() + chrono::Duration::days(7) {
        crate::linkedin::refresh_linkedin_token(
            &app_state.db,
            &app_state.encrypt_config,
            linkedin_config,
            &row,
        )
        .await?;
        tracing::info!("Refreshed LinkedIn access token");
    }

    Ok(())
}
