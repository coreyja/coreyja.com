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

    sqlx::migrate!().run(&pool).await.into_diagnostic()?;

    Ok(pool)
}