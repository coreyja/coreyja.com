use serde::{de::DeserializeOwned, Serialize};
use tracing::instrument;

use crate::AppState;
use miette::IntoDiagnostic;

pub mod sponsors;
pub mod youtube_videos;

#[async_trait::async_trait]
pub(crate) trait Job:
    Serialize + DeserializeOwned + Send + Sync + std::fmt::Debug + Clone + 'static
{
    const NAME: &'static str;

    async fn run(&self, app_state: AppState) -> miette::Result<()>;

    #[instrument(name = "jobs.run_from_value", skip(app_state), fields(job.name = Self::NAME), err)]
    async fn run_from_value(value: serde_json::Value, app_state: AppState) -> miette::Result<()> {
        let job: Self = serde_json::from_value(value).into_diagnostic()?;

        job.run(app_state).await
    }

    #[instrument(name = "jobs.enqueue", skip(app_state), fields(job.name = Self::NAME), err)]
    async fn enqueue(self, app_state: AppState) -> miette::Result<()> {
        sqlx::query!(
            "
        INSERT INTO jobs (job_id, name, payload, priority, run_at, created_at, context)
        VALUES ($1, $2, $3, $4, $5, $6, $7)",
            uuid::Uuid::new_v4(),
            Self::NAME,
            serde_json::to_value(self).into_diagnostic()?,
            0,
            chrono::Utc::now(),
            chrono::Utc::now(),
            ""
        )
        .execute(&app_state.db)
        .await
        .into_diagnostic()?;

        Ok(())
    }
}

pub mod meta;
