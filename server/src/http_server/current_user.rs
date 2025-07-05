use axum::{
    body::Body,
    extract::FromRequestParts,
    http::{Response, StatusCode},
    response::{IntoResponse, Redirect},
};
use cja::server::session::Session;
use db::users::UserFromDB;

use crate::{github::GithubLink, http_server::auth::session::DBSession, AppState};

pub struct CurrentUser {
    pub user: UserFromDB,
    pub session: DBSession,
    pub github_link: GithubLink,
}

pub enum CurrentUserError {
    SessionRedirect(Redirect),
    Unauthorized(StatusCode),
    DBError(sqlx::Error),
}

impl From<Redirect> for CurrentUserError {
    fn from(value: Redirect) -> Self {
        Self::SessionRedirect(value)
    }
}

impl From<sqlx::Error> for CurrentUserError {
    fn from(e: sqlx::Error) -> Self {
        Self::DBError(e)
    }
}

impl From<StatusCode> for CurrentUserError {
    fn from(e: StatusCode) -> Self {
        Self::Unauthorized(e)
    }
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = CurrentUserError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Session(session) = Session::<DBSession>::from_request_parts(parts, state).await?;

        let user = sqlx::query_as!(
            UserFromDB,
            r#"
        SELECT *
        FROM Users
        WHERE user_id = $1 and user_id is not null
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
            Self::Unauthorized(e) => e.into_response(),
        }
    }
}
