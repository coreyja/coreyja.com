use async_trait::async_trait;
use axum::{extract::FromRequestParts, http, response::Redirect};
use cja::server::session::{DBSession, SessionRedirect};

use crate::{github::GithubLink, AppState};

#[derive(Debug, Clone)]
pub struct AdminUser {
    pub session: DBSession,
    pub github_link: GithubLink,
}

const COREYJA_PERSONAL_GITHUB_ID: &str = "MDQ6VXNlcjk2NDc3MQ==";

#[async_trait]
impl FromRequestParts<AppState> for AdminUser {
    type Rejection = cja::server::session::SessionRedirect;

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
            Ok(None) => Err(SessionRedirect::temporary("/")),
            Err(e) => {
                sentry::capture_error(&e);

                Err(SessionRedirect::temporary("/"))
            }
        }
    }
}
