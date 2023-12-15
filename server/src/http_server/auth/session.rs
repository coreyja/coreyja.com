use async_trait::async_trait;
use axum::{extract::FromRequestParts, http, response::Redirect};
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;

use crate::{github::GithubLink, AppState};

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

#[derive(Debug, Clone)]
pub struct AdminUser {
    pub session: DBSession,
    pub github_link: GithubLink,
}

const COREYJA_PERSONAL_GITHUB_ID: &str = "MDQ6VXNlcjk2NDc3MQ==";

#[async_trait]
impl FromRequestParts<AppState> for AdminUser {
    type Rejection = axum::response::Redirect;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let db_session = DBSession::from_request_parts(parts, state).await?;

        let github_link = sqlx::query_as!(
            GithubLink,
            r#"
            SELECT *
            FROM GithubLinks
            WHERE user_id = $1 AND external_github_id = $2
            "#,
            db_session.user_id,
            COREYJA_PERSONAL_GITHUB_ID
        )
        .fetch_optional(&state.db)
        .await;

        match github_link {
            Ok(Some(github_link)) => Ok(AdminUser {
                session: db_session,
                github_link,
            }),
            Ok(None) => Err(Redirect::temporary("/")),
            Err(e) => {
                sentry::capture_error(&e);

                Err(Redirect::temporary("/"))
            }
        }
    }
}
