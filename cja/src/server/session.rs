use async_trait::async_trait;
use axum::{body::Body, extract::FromRequestParts, response::IntoResponse};
use http::{header, Response};
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::app_state::AppState as AS;

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::FromRow)]
pub struct DBSession {
    pub session_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionRedirect {
    location: String,
}

impl SessionRedirect {
    pub fn temporary(location: &str) -> Self {
        Self {
            location: location.to_string(),
        }
    }
}

impl IntoResponse for SessionRedirect {
    fn into_response(self) -> Response<Body> {
        (
            http::status::StatusCode::TEMPORARY_REDIRECT,
            [(header::LOCATION, self.location)],
        )
            .into_response()
    }
}

#[async_trait]
impl<AppState: AS> FromRequestParts<AppState> for DBSession {
    type Rejection = SessionRedirect;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookies = Cookies::from_request_parts(parts, state)
            .await
            .map_err(|(_, msg)| {
                tracing::error!("Failed to get cookies: {msg}");

                SessionRedirect::temporary("/login")
            })?;

        let private = cookies.private(state.cookie_key());

        let session_cookie = private.get("session_id");

        let Some(session_cookie) = session_cookie else {
            let return_to_path = parts
                .uri
                .path_and_query()
                .map_or("/", http::uri::PathAndQuery::as_str);

            Err(SessionRedirect::temporary(&format!(
                "/login?return_to={return_to_path}"
            )))?
        };

        let session_id = session_cookie.value().to_string();
        let Ok(session_id) = uuid::Uuid::parse_str(&session_id) else {
            tracing::error!("Failed to parse session id: {session_id}");

            Err(SessionRedirect::temporary("/login"))?
        };

        let session = sqlx::query_as::<_, DBSession>(
            r"
        SELECT *
        FROM Sessions
        WHERE session_id = $1
        ",
        )
        .bind(session_id)
        .fetch_one(state.db())
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch session: {e}");

            SessionRedirect::temporary("/login")
        })?;

        Ok(session)
    }
}

impl DBSession {
    pub async fn create<AppState: AS>(
        user_id: Uuid,
        app_state: &AppState,
        cookies: &Cookies,
    ) -> color_eyre::Result<Self> {
        let session = sqlx::query_as::<_, DBSession>(
            r"
        INSERT INTO Sessions (session_id, user_id)
        VALUES ($1, $2)
        RETURNING *
        ",
        )
        .bind(uuid::Uuid::new_v4())
        .bind(user_id)
        .fetch_one(app_state.db())
        .await?;

        let private = cookies.private(app_state.cookie_key());

        let session_cookie =
            tower_cookies::Cookie::build(("session_id", session.session_id.to_string()))
                .path("/")
                .http_only(true)
                .secure(true)
                .expires(None);
        private.add(session_cookie.into());

        Ok(session)
    }
}
