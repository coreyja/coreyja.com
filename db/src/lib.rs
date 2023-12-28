use miette::{IntoDiagnostic, Result};
use sqlx::postgres::PgPoolOptions;

pub mod twitch_chatters;
pub mod users;

pub use sqlx;
pub use sqlx::PgPool;

pub async fn setup_db_pool() -> Result<PgPool> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .into_diagnostic()?;

    const MIGRATION_LOCK_ID: i64 = 0xDB_DB_DB_DB_DB_DB_DB;
    sqlx::query!("SELECT pg_advisory_lock($1)", MIGRATION_LOCK_ID)
        .execute(&pool)
        .await
        .into_diagnostic()?;

    sqlx::migrate!().run(&pool).await.into_diagnostic()?;

    let unlock_result = sqlx::query!("SELECT pg_advisory_unlock($1)", MIGRATION_LOCK_ID)
        .fetch_one(&pool)
        .await
        .into_diagnostic()?
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
