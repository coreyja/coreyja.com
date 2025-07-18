use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ToolSuggestion {
    pub suggestion_id: Uuid,
    pub name: String,
    pub description: String,
    pub examples: serde_json::Value,
    pub status: String,
    pub linear_ticket_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ToolSuggestion {
    pub async fn create(
        pool: &PgPool,
        name: String,
        description: String,
        examples: serde_json::Value,
    ) -> color_eyre::Result<Self> {
        let suggestion = sqlx::query_as!(
            Self,
            r#"
            INSERT INTO tool_suggestions (name, description, examples)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            name,
            description,
            examples
        )
        .fetch_one(pool)
        .await?;

        Ok(suggestion)
    }

    pub async fn list_pending(pool: &PgPool) -> color_eyre::Result<Vec<Self>> {
        let suggestions = sqlx::query_as!(
            Self,
            r#"
            SELECT * FROM tool_suggestions
            WHERE status = 'pending'
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(suggestions)
    }

    pub async fn get_by_id(pool: &PgPool, suggestion_id: Uuid) -> color_eyre::Result<Option<Self>> {
        let suggestion = sqlx::query_as!(
            Self,
            r#"
            SELECT * FROM tool_suggestions
            WHERE suggestion_id = $1
            "#,
            suggestion_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(suggestion)
    }

    pub async fn dismiss(
        pool: &PgPool,
        suggestion_id: Uuid,
        linear_ticket_id: String,
    ) -> color_eyre::Result<Self> {
        let suggestion = sqlx::query_as!(
            Self,
            r#"
            UPDATE tool_suggestions
            SET status = 'dismissed', linear_ticket_id = $2
            WHERE suggestion_id = $1
            RETURNING *
            "#,
            suggestion_id,
            linear_ticket_id
        )
        .fetch_one(pool)
        .await?;

        Ok(suggestion)
    }

    pub async fn skip(pool: &PgPool, suggestion_id: Uuid) -> color_eyre::Result<Self> {
        let suggestion = sqlx::query_as!(
            Self,
            r#"
            UPDATE tool_suggestions
            SET status = 'skipped'
            WHERE suggestion_id = $1
            RETURNING *
            "#,
            suggestion_id
        )
        .fetch_one(pool)
        .await?;

        Ok(suggestion)
    }
}
