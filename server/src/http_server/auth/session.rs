use axum::{
    extract::FromRequestParts,
    http::{self, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use cja::server::session::{AppSession, CJASession, Session};
use color_eyre::eyre::{eyre, Context};
use uuid::Uuid;

use crate::{github::GithubLink, AppState};

#[derive(Debug, Clone)]
pub struct DBSession {
    pub user_id: Option<Uuid>,
    cja_session: CJASession,
}

#[async_trait::async_trait]
impl AppSession for DBSession {
    async fn from_db(pool: &sqlx::PgPool, session_id: uuid::Uuid) -> crate::Result<Self> {
        let session = sqlx::query!("SELECT * FROM Sessions where session_id = $1", session_id)
            .fetch_one(pool)
            .await
            .wrap_err_with(|| eyre!("Failed to fetch session from database"))?;
        let inner = CJASession {
            session_id: session.session_id,
            created_at: session.created_at,
            updated_at: session.updated_at,
        };
        Ok(DBSession {
            user_id: session.user_id,
            cja_session: inner,
        })
    }

    /// Create a new session in the database.
    ///
    /// This method should insert a new session record with default values
    /// and return the created session.
    async fn create(pool: &sqlx::PgPool) -> crate::Result<Self> {
        let session = sqlx::query!("INSERT INTO Sessions DEFAULT VALUES RETURNING *")
            .fetch_one(pool)
            .await?;

        let inner = CJASession {
            session_id: session.session_id,
            created_at: session.created_at,
            updated_at: session.updated_at,
        };
        Ok(DBSession {
            user_id: session.user_id,
            cja_session: inner,
        })
    }

    /// Create a session instance from the inner `CJASession`.
    ///
    /// This is used internally when reconstructing sessions. Custom fields
    /// should be initialized with default values.
    fn from_inner(inner: CJASession) -> Self {
        DBSession {
            user_id: None,
            cja_session: inner,
        }
    }

    /// Get a reference to the inner `CJASession`.
    ///
    /// This provides access to the core session fields like ID and timestamps.
    fn inner(&self) -> &CJASession {
        &self.cja_session
    }
}

#[derive(Clone)]
pub struct AdminUser {
    pub session: Session<DBSession>,
    pub user_id: Uuid,
    pub github_link: GithubLink,
}

const COREYJA_PERSONAL_GITHUB_ID: &str = "MDQ6VXNlcjk2NDc3MQ==";

impl GithubLink {
    pub fn is_coreyja(&self) -> bool {
        self.external_github_id == COREYJA_PERSONAL_GITHUB_ID
    }
}

pub async fn is_admin_user(user_id: Uuid, state: &AppState) -> cja::Result<bool> {
    let github_link = sqlx::query_as!(
        GithubLink,
        r#"SELECT * FROM GithubLinks WHERE user_id = $1"#,
        user_id,
    )
    .fetch_optional(&state.db)
    .await?;

    let Some(github_link) = github_link else {
        return Ok(false);
    };

    Ok(github_link.is_coreyja())
}

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let db_session = Session::<DBSession>::from_request_parts(parts, state)
            .await
            .map_err(|e| e.into_response())?;
        let Some(user_id) = db_session.0.user_id else {
            // User not authenticated - redirect to login
            return Err(Redirect::temporary("/login").into_response());
        };

        let github_link = sqlx::query_as!(
            GithubLink,
            r#"
            SELECT *
            FROM GithubLinks
            WHERE user_id = $1 AND external_github_id = $2
            "#,
            user_id,
            COREYJA_PERSONAL_GITHUB_ID
        )
        .fetch_optional(&state.db)
        .await;

        match github_link {
            Ok(Some(github_link)) => Ok(AdminUser {
                session: db_session,
                user_id,
                github_link,
            }),
            Ok(None) => Err(StatusCode::UNAUTHORIZED.into_response()),
            Err(e) => {
                sentry::capture_error(&e);

                Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
            }
        }
    }
}
