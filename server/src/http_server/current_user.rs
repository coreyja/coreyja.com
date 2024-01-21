use async_trait::async_trait;
use axum::{
    body::Body,
    extract::FromRequestParts,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use cja::server::session::{self, DBSession, SessionRedirect};
use db::users::UserFromDB;

use crate::{github::GithubLink, AppState};

pub struct CurrentUser {
    pub user: UserFromDB,
    pub session: DBSession,
    pub github_link: GithubLink,
}

pub enum CurrentUserError {
    SessionRedirect(session::SessionRedirect),
    DBError(sqlx::Error),
}

impl From<SessionRedirect> for CurrentUserError {
    fn from(value: SessionRedirect) -> Self {
        Self::SessionRedirect(value)
    }
}

impl From<sqlx::Error> for CurrentUserError {
    fn from(e: sqlx::Error) -> Self {
        Self::DBError(e)
    }
}

#[async_trait]
impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = CurrentUserError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let session = DBSession::from_request_parts(parts, state).await?;

        let user = sqlx::query_as!(
            UserFromDB,
            r#"
        SELECT *
        FROM Users
        WHERE user_id = $1
        "#,
            session.user_id
        )
        .fetch_one(&state.db)
        .await?;

        let github_link = sqlx::query_as!(
            GithubLink,
            r#"
            SELECT *
            FROM GithubLinks
            WHERE user_id = $1
            "#,
            session.user_id,
        )
        .fetch_one(&state.db)
        .await?;

        Ok(Self {
            user,
            session,
            github_link,
        })
    }
}

impl IntoResponse for CurrentUserError {
    fn into_response(self) -> Response<Body> {
        match self {
            Self::SessionRedirect(session_redirect) => session_redirect.into_response(),
            Self::DBError(e) => {
                tracing::error!(error = %e, "CurrentUserError");

                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
