use crate::app_state::AppState as AS;
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;
use tracing::instrument;

pub mod registry;

#[derive(Debug, Error)]
pub enum EnqueueError {
    #[error("SqlxError: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("SerdeJsonError: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
}

#[async_trait::async_trait]
pub trait Job<AppState: AS>:
    Serialize + DeserializeOwned + Send + Sync + std::fmt::Debug + Clone + 'static
{
    const NAME: &'static str;

    async fn run(&self, app_state: AppState) -> color_eyre::Result<()>;

    #[instrument(name = "jobs.run_from_value", skip(app_state), fields(job.name = Self::NAME), err)]
    async fn run_from_value(
        value: serde_json::Value,
        app_state: AppState,
    ) -> color_eyre::Result<()> {
        let job: Self = serde_json::from_value(value)?;

        job.run(app_state).await
    }

    #[instrument(name = "jobs.enqueue", skip(app_state), fields(job.name = Self::NAME), err)]
    async fn enqueue(self, app_state: AppState, context: String) -> Result<(), EnqueueError> {
        sqlx::query(
            "
        INSERT INTO jobs (job_id, name, payload, priority, run_at, created_at, context)
        VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(uuid::Uuid::new_v4())
        .bind(Self::NAME)
        .bind(serde_json::to_value(self)?)
        .bind(0)
        .bind(chrono::Utc::now())
        .bind(chrono::Utc::now())
        .bind(context)
        .execute(app_state.db())
        .await?;

        Ok(())
    }
}

pub mod worker;
