use miette::{Diagnostic, IntoDiagnostic, Result};
use thiserror::Error;
use tracing::Span;

use crate::state::AppState;

use super::{sponsors::RefreshSponsors, Job};

#[derive(Debug, Clone)]
pub struct JobId(uuid::Uuid);

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.to_string().fmt(f)
    }
}

pub(crate) type RunJobResult = Result<RunJobSuccess, JobError>;

#[derive(Debug)]
pub enum RunJobSuccess {
    JobRan(JobFromDB),
    NoRunnableJobInQueue,
}

#[derive(Debug)]
struct JobFromDB {
    job_id: uuid::Uuid,
    name: String,
    payload: serde_json::Value,
    priority: i32,
    run_at: chrono::DateTime<chrono::Utc>,
    created_at: chrono::DateTime<chrono::Utc>,
    context: String,
}

#[derive(Debug, Diagnostic, Error)]
#[error("JobError(id:${1}: ${0}")]
pub(crate) struct JobError(JobId, miette::Report);

struct Worker {
    id: uuid::Uuid,
    state: AppState,
}

impl Worker {
    fn new(state: AppState) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            state,
        }
    }

    #[tracing::instrument(
        name = "worker.run_job",
        skip(self, job),
        fields(
            job.id = %job.job_id,
            job.name = job.name,
            job.priority = job.priority,
            job.run_at = %job.run_at,
            job.created_at = %job.created_at,
            job.context = job.context,
            worker.id = %self.id,
        )
        err,
    )]
    async fn run_job(&self, job: &JobFromDB) -> Result<()> {
        let payload = job.payload.clone();

        match job.name.as_str() {
            "RefreshSponsors" => RefreshSponsors::run_from_value(payload, self.state.clone()).await,
            "RefreshVideos" => {
                super::youtube_videos::RefreshVideos::run_from_value(payload, self.state.clone())
                    .await
            }
            _ => Err(miette::miette!("Unknown job type: {}", job.name)),
        }
    }

    pub(crate) async fn run_next_job(&self) -> Result<RunJobResult> {
        let job = self.fetch_next_job().await?;

        let Some(job) = job else {
            return Ok(Ok(RunJobSuccess::NoRunnableJobInQueue));
        };

        let job_id = JobId(job.job_id);
        let job_result = self.run_job(&job).await;

        if let Err(e) = job_result {
            sqlx::query!(
                "
              UPDATE jobs
              SET locked_by = NULL, locked_at = NULL, run_at = now() + interval '60 seconds'
              WHERE job_id = $1 AND locked_by = $2
                  ",
                job.job_id,
                self.id.to_string()
            )
            .execute(&self.state.db)
            .await
            .into_diagnostic()?;

            return Ok(Err(JobError(job_id, e)));
        }

        sqlx::query!(
            "
                DELETE FROM jobs
                WHERE job_id = $1 AND locked_by = $2
                ",
            job.job_id,
            self.id.to_string()
        )
        .execute(&self.state.db)
        .await
        .into_diagnostic()?;

        Ok(Ok(RunJobSuccess::JobRan(job)))
    }

    #[tracing::instrument(
        name = "worker.fetch_next_job",
        skip(self),
        fields(
            worker.id = %self.id,
            job.id,
            job.name,
        ),
        err,
    )]
    async fn fetch_next_job(&self) -> miette::Result<Option<JobFromDB>> {
        let now = chrono::Utc::now();
        let job = sqlx::query_as!(
            JobFromDB,
            "
            UPDATE jobs
            SET LOCKED_BY = $1, LOCKED_AT = $2
            WHERE job_id = (
                SELECT job_id
                FROM jobs
                WHERE run_at <= NOW() AND locked_by IS NULL
                ORDER BY priority DESC, created_at ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING job_id, name, payload, priority, run_at, created_at, context
            ",
            self.id.to_string(),
            now,
        )
        .fetch_optional(&self.state.db)
        .await
        .into_diagnostic()?;

        if let Some(job) = &job {
            let span = Span::current();
            span.record("job.id", job.job_id.to_string());
            span.record("job.name", &job.name);
        }

        Ok(job)
    }

    #[tracing::instrument(
        name = "worker.tick",
        skip(self),
        fields(
            worker.id = %self.id,
        ),
    )]
    async fn tick(&self) -> miette::Result<()> {
        let result = self.run_next_job().await?;

        match result {
            Ok(RunJobSuccess::JobRan(job)) => {
                tracing::info!(worker.id =% self.id, job_id =% job.job_id, "Job Ran");
            }
            Ok(RunJobSuccess::NoRunnableJobInQueue) => {
                let duration = std::time::Duration::from_secs(5);
                tracing::debug!(worker.id =% self.id, ?duration, "No Job to Run, sleeping for requested duration");

                tokio::time::sleep(duration).await;
            }
            Err(job_error) => {
                sentry::capture_error(&job_error);
                tracing::error!(
                    worker.id =% self.id,
                    job_id =% job_error.0.0,
                    error_msg =% job_error.1,
                    "Job Errored"
                );
            }
        }

        Ok(())
    }
}

pub(crate) async fn job_worker(app_state: crate::AppState) -> Result<()> {
    let worker = Worker::new(app_state);

    loop {
        worker.tick().await?;
    }
}
