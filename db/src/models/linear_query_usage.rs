use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LinearQueryUsage {
    pub id: Uuid,
    pub query_id: Uuid,
    pub executed_at: DateTime<Utc>,
    pub variables: Option<JsonValue>,
    pub success: bool,
    pub error_message: Option<String>,
}

impl LinearQueryUsage {
    pub async fn create(
        conn: &mut sqlx::PgConnection,
        query_id: Uuid,
        variables: Option<JsonValue>,
        success: bool,
        error_message: Option<String>,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            LinearQueryUsage,
            r#"
            INSERT INTO linear_query_usage (
                query_id,
                variables,
                success,
                error_message
            )
            VALUES ($1, $2, $3, $4)
            RETURNING
                id,
                query_id,
                executed_at,
                variables,
                success,
                error_message
            "#,
            query_id,
            variables,
            success,
            error_message,
        )
        .fetch_one(conn)
        .await
    }

    pub async fn find_by_query_id(
        conn: &mut sqlx::PgConnection,
        query_id: Uuid,
        limit: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            LinearQueryUsage,
            r#"
            SELECT
                id,
                query_id,
                executed_at,
                variables,
                success,
                error_message
            FROM linear_query_usage
            WHERE query_id = $1
            ORDER BY executed_at DESC
            LIMIT $2
            "#,
            query_id,
            limit
        )
        .fetch_all(conn)
        .await
    }

    pub async fn get_stats_for_query(
        conn: &mut sqlx::PgConnection,
        query_id: Uuid,
    ) -> Result<QueryUsageStats, sqlx::Error> {
        let stats = sqlx::query!(
            r#"
            SELECT
                COUNT(*) as total_executions,
                COUNT(CASE WHEN success = true THEN 1 END) as successful_executions,
                COUNT(CASE WHEN success = false THEN 1 END) as failed_executions,
                MAX(executed_at) as last_executed_at
            FROM linear_query_usage
            WHERE query_id = $1
            "#,
            query_id
        )
        .fetch_one(conn)
        .await?;

        Ok(QueryUsageStats {
            total_executions: stats.total_executions.unwrap_or(0),
            successful_executions: stats.successful_executions.unwrap_or(0),
            failed_executions: stats.failed_executions.unwrap_or(0),
            last_executed_at: stats.last_executed_at,
        })
    }

    pub async fn cleanup_old_usage(
        conn: &mut sqlx::PgConnection,
        days_to_keep: i32,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM linear_query_usage
            WHERE executed_at < NOW() - INTERVAL '1 day' * $1
            "#,
            f64::from(days_to_keep)
        )
        .execute(conn)
        .await?;

        Ok(result.rows_affected())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryUsageStats {
    pub total_executions: i64,
    pub successful_executions: i64,
    pub failed_executions: i64,
    pub last_executed_at: Option<DateTime<Utc>>,
}
