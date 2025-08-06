use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LinearThreadMetadata {
    pub thread_id: Uuid,
    pub session_id: String,
    pub workspace_id: String,
    pub issue_id: Option<String>,
    pub issue_title: Option<String>,
    pub project_id: Option<String>,
    pub team_id: Option<String>,
    pub created_by_user_id: String,
    pub session_status: String,
    pub last_activity_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl LinearThreadMetadata {
    pub async fn create(
        pool: &PgPool,
        thread_id: Uuid,
        session_id: String,
        workspace_id: String,
        issue_id: Option<String>,
        issue_title: Option<String>,
        project_id: Option<String>,
        team_id: Option<String>,
        created_by_user_id: String,
    ) -> Result<Self, sqlx::Error> {
        let metadata = sqlx::query_as!(
            LinearThreadMetadata,
            r#"
            INSERT INTO linear_thread_metadata (
                thread_id, session_id, workspace_id, issue_id, issue_title,
                project_id, team_id, created_by_user_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
            thread_id,
            session_id,
            workspace_id,
            issue_id,
            issue_title,
            project_id,
            team_id,
            created_by_user_id,
        )
        .fetch_one(pool)
        .await?;

        Ok(metadata)
    }

    pub async fn find_by_session_id(
        pool: &PgPool,
        session_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let metadata = sqlx::query_as!(
            LinearThreadMetadata,
            r#"
            SELECT * FROM linear_thread_metadata
            WHERE session_id = $1
            "#,
            session_id,
        )
        .fetch_optional(pool)
        .await?;

        Ok(metadata)
    }

    pub async fn find_by_thread_id(
        pool: &PgPool,
        thread_id: Uuid,
    ) -> Result<Option<Self>, sqlx::Error> {
        let metadata = sqlx::query_as!(
            LinearThreadMetadata,
            r#"
            SELECT * FROM linear_thread_metadata
            WHERE thread_id = $1
            "#,
            thread_id,
        )
        .fetch_optional(pool)
        .await?;

        Ok(metadata)
    }

    pub async fn find_by_issue_id(pool: &PgPool, issue_id: &str) -> Result<Vec<Self>, sqlx::Error> {
        let metadata = sqlx::query_as!(
            LinearThreadMetadata,
            r#"
            SELECT * FROM linear_thread_metadata
            WHERE issue_id = $1
            ORDER BY created_at DESC
            "#,
            issue_id,
        )
        .fetch_all(pool)
        .await?;

        Ok(metadata)
    }

    pub async fn update_session_status(
        pool: &PgPool,
        thread_id: Uuid,
        status: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let metadata = sqlx::query_as!(
            LinearThreadMetadata,
            r#"
            UPDATE linear_thread_metadata
            SET session_status = $2, last_activity_at = NOW()
            WHERE thread_id = $1
            RETURNING *
            "#,
            thread_id,
            status,
        )
        .fetch_optional(pool)
        .await?;

        Ok(metadata)
    }

    pub async fn update_last_activity(
        pool: &PgPool,
        thread_id: Uuid,
    ) -> Result<Option<Self>, sqlx::Error> {
        let metadata = sqlx::query_as!(
            LinearThreadMetadata,
            r#"
            UPDATE linear_thread_metadata
            SET last_activity_at = NOW()
            WHERE thread_id = $1
            RETURNING *
            "#,
            thread_id,
        )
        .fetch_optional(pool)
        .await?;

        Ok(metadata)
    }

    pub async fn find_active_sessions(
        pool: &PgPool,
        workspace_id: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let metadata = sqlx::query_as!(
            LinearThreadMetadata,
            r#"
            SELECT * FROM linear_thread_metadata
            WHERE workspace_id = $1 
                AND session_status NOT IN ('complete', 'error')
            ORDER BY last_activity_at DESC
            "#,
            workspace_id,
        )
        .fetch_all(pool)
        .await?;

        Ok(metadata)
    }

    pub async fn find_timed_out_sessions(
        pool: &PgPool,
        timeout_minutes: i32,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let metadata = sqlx::query_as!(
            LinearThreadMetadata,
            r#"
            SELECT * FROM linear_thread_metadata
            WHERE session_status IN ('active', 'awaitingInput')
                AND last_activity_at < NOW() - INTERVAL '1 minute' * $1
            "#,
            f64::from(timeout_minutes),
        )
        .fetch_all(pool)
        .await?;

        Ok(metadata)
    }
}
