use chrono::{DateTime, Utc};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;

pub mod agentic_threads;
pub mod discord_threads;
pub mod linear_threads;
pub mod models;
pub mod tool_suggestions;
pub mod twitch_chatters;
pub mod users;

pub use sqlx;
pub use sqlx::PgPool;
use uuid::Uuid;

#[tracing::instrument(err)]
pub async fn setup_db_pool() -> Result<PgPool> {
    const MIGRATION_LOCK_ID: i64 = 0xDB_DB_DB_DB_DB_DB_DB;

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::query!("SELECT pg_advisory_lock($1)", MIGRATION_LOCK_ID)
        .execute(&pool)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    let unlock_result = sqlx::query!("SELECT pg_advisory_unlock($1)", MIGRATION_LOCK_ID)
        .fetch_one(&pool)
        .await?
        .pg_advisory_unlock;

    match unlock_result {
        Some(b) => {
            if b {
                tracing::info!("Migration lock unlocked");
            } else {
                tracing::info!("Failed to unlock migration lock");
            }
        }
        None => panic!("Failed to unlock migration lock"),
    }

    Ok(pool)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordChannel {
    pub discord_channel_id: Uuid,
    pub channel_name: Option<String>,
    pub channel_topic: Option<String>,
    pub channel_id: String,
    pub purpose: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
