use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LinearSavedQuery {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub query: String,
    pub variables_schema: Option<JsonValue>,
    pub tags: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
}

impl LinearSavedQuery {
    pub async fn create(
        conn: &mut sqlx::PgConnection,
        name: String,
        description: Option<String>,
        query: String,
        variables_schema: Option<JsonValue>,
        tags: Option<Vec<String>>,
        created_by: Option<String>,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            LinearSavedQuery,
            r#"
            INSERT INTO linear_saved_queries (name, description, query, variables_schema, tags, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING
                id,
                name,
                description,
                query,
                variables_schema,
                tags,
                created_at,
                updated_at,
                created_by
            "#,
            name,
            description,
            query,
            variables_schema,
            tags.as_deref(),
            created_by
        )
        .fetch_one(conn)
        .await
    }

    pub async fn find_by_id(
        conn: &mut sqlx::PgConnection,
        id: Uuid,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            LinearSavedQuery,
            r#"
            SELECT
                id,
                name,
                description,
                query,
                variables_schema,
                tags,
                created_at,
                updated_at,
                created_by
            FROM linear_saved_queries
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(conn)
        .await
    }

    pub async fn find_by_name(
        conn: &mut sqlx::PgConnection,
        name: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            LinearSavedQuery,
            r#"
            SELECT
                id,
                name,
                description,
                query,
                variables_schema,
                tags,
                created_at,
                updated_at,
                created_by
            FROM linear_saved_queries
            WHERE name = $1
            "#,
            name
        )
        .fetch_optional(conn)
        .await
    }

    pub async fn search(
        conn: &mut sqlx::PgConnection,
        search_term: Option<&str>,
        tags: Option<&[String]>,
        limit: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        if search_term.is_some() && tags.is_some() {
            sqlx::query_as!(
                LinearSavedQuery,
                r#"
                SELECT
                    id,
                    name,
                    description,
                    query,
                    variables_schema,
                    tags,
                    created_at,
                    updated_at,
                    created_by
                FROM linear_saved_queries
                WHERE (name ILIKE $1 OR description ILIKE $1)
                    AND tags && $2
                ORDER BY created_at DESC
                LIMIT $3
                "#,
                format!("%{}%", search_term.unwrap()),
                tags.unwrap(),
                limit
            )
            .fetch_all(conn)
            .await
        } else if let Some(term) = search_term {
            sqlx::query_as!(
                LinearSavedQuery,
                r#"
                SELECT
                    id,
                    name,
                    description,
                    query,
                    variables_schema,
                    tags,
                    created_at,
                    updated_at,
                    created_by
                FROM linear_saved_queries
                WHERE name ILIKE $1 OR description ILIKE $1
                ORDER BY created_at DESC
                LIMIT $2
                "#,
                format!("%{}%", term),
                limit
            )
            .fetch_all(conn)
            .await
        } else if let Some(tag_list) = tags {
            sqlx::query_as!(
                LinearSavedQuery,
                r#"
                SELECT
                    id,
                    name,
                    description,
                    query,
                    variables_schema,
                    tags,
                    created_at,
                    updated_at,
                    created_by
                FROM linear_saved_queries
                WHERE tags && $1
                ORDER BY created_at DESC
                LIMIT $2
                "#,
                tag_list,
                limit
            )
            .fetch_all(conn)
            .await
        } else {
            sqlx::query_as!(
                LinearSavedQuery,
                r#"
                SELECT
                    id,
                    name,
                    description,
                    query,
                    variables_schema,
                    tags,
                    created_at,
                    updated_at,
                    created_by
                FROM linear_saved_queries
                ORDER BY created_at DESC
                LIMIT $1
                "#,
                limit
            )
            .fetch_all(conn)
            .await
        }
    }

    pub async fn update(&self, conn: &mut sqlx::PgConnection) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            LinearSavedQuery,
            r#"
            UPDATE linear_saved_queries
            SET
                name = $2,
                description = $3,
                query = $4,
                variables_schema = $5,
                tags = $6,
                updated_at = NOW()
            WHERE id = $1
            RETURNING
                id,
                name,
                description,
                query,
                variables_schema,
                tags,
                created_at,
                updated_at,
                created_by
            "#,
            self.id,
            self.name,
            self.description,
            self.query,
            self.variables_schema,
            self.tags.as_deref()
        )
        .fetch_one(conn)
        .await
    }

    pub async fn delete(conn: &mut sqlx::PgConnection, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM linear_saved_queries WHERE id = $1", id)
            .execute(conn)
            .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearSavedQueryWithStats {
    pub query: LinearSavedQuery,
    pub last_used: Option<DateTime<Utc>>,
    pub use_count: i64,
}

impl LinearSavedQueryWithStats {
    pub async fn find_with_stats(
        conn: &mut sqlx::PgConnection,
        search_term: Option<&str>,
        tags: Option<&[String]>,
        limit: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        // First get the queries
        let queries = LinearSavedQuery::search(conn, search_term, tags, limit).await?;

        // Then get stats for each
        let mut results = Vec::new();
        for query in queries {
            let stats = sqlx::query!(
                r#"
                SELECT
                    MAX(executed_at) as last_used,
                    COUNT(*) as use_count
                FROM linear_query_usage
                WHERE query_id = $1 AND success = true
                "#,
                query.id
            )
            .fetch_one(&mut *conn)
            .await?;

            results.push(LinearSavedQueryWithStats {
                query,
                last_used: stats.last_used,
                use_count: stats.use_count.unwrap_or(0),
            });
        }

        Ok(results)
    }
}
