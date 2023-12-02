use async_trait::async_trait;
use axum::{extract::FromRequestParts, http, response::Redirect};
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;

use crate::AppState;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DBSession {
    pub session_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
impl FromRequestParts<AppState> for DBSession {
    type Rejection = axum::response::Redirect;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookies = Cookies::from_request_parts(parts, state)
            .await
            .map_err(|(_, msg)| {
                sentry::capture_message(
                    &format!("Failed to get cookies: {msg}"),
                    sentry::Level::Error,
                );

                Redirect::temporary("/login")
            })?;

        let private = cookies.private(&state.cookie_key.0);

        let session_cookie = private.get("session_id");

        let Some(session_cookie) = session_cookie else {
            Err(Redirect::temporary("/login"))?
        };
        let session_id = session_cookie.value().to_string();
        let Ok(session_id) = uuid::Uuid::parse_str(&session_id) else {
            sentry::capture_message(
                &format!("Failed to parse session id: {session_id}"),
                sentry::Level::Error,
            );
            Err(Redirect::temporary("/login"))?
        };

        let session = sqlx::query_as!(
            DBSession,
            r#"
        SELECT *
        FROM Sessions
        WHERE session_id = $1
        "#,
            &session_id
        )
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            sentry::capture_error(&e);

            Redirect::temporary("/login")
        })?;

        Ok(session)
    }
}
