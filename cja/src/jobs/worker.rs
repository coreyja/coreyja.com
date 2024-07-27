use thiserror::Error;
use tracing::Span;

use crate::app_state::AppState as AS;

use super::registry::JobRegistry;

pub(super) type RunJobResult = Result<RunJobSuccess, JobError>;

#[derive(Debug)]
pub(super) struct RunJobSuccess(JobFromDB);

#[derive(Debug, sqlx::FromRow)]
pub struct JobFromDB {
    pub job_id: uuid::Uuid,
    pub name: String,
    pub payload: serde_json::Value,
    pub priority: i32,
    pub run_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub context: String,
}

#[derive(Debug, Error)]
#[error("JobError(id:${}) ${1}", self.0.job_id)]
pub(crate) struct JobError(JobFromDB, color_eyre::Report);

struct Worker<AppState: AS, R: JobRegistry<AppState>> {
    id: uuid::Uuid,
    state: AppState,
    registry: R,
}

impl<AppState: AS, R: JobRegistry<AppState>> Worker<AppState, R> {
    fn new(state: AppState, registry: R) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            state,
            registry,
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
    async fn run_job(&self, job: &JobFromDB) -> color_eyre::Result<()> {
        self.registry.run_job(job, self.state.clone()).await
    }

    pub(crate) async fn run_next_job(&self, job: JobFromDB) -> color_eyre::Result<RunJobResult> {
        let job_result = self.run_job(&job).await;

        if let Err(e) = job_result {
            sqlx::query(
                "
              UPDATE jobs
              SET locked_by = NULL, locked_at = NULL, run_at = now() + interval '60 seconds'
              WHERE job_id = $1 AND locked_by = $2
                  ",
            )
            .bind(job.job_id)
            .bind(self.id.to_string())
            .execute(self.state.db())
            .await?;

            return Ok(Err(JobError(job, e)));
        }

        sqlx::query(
            "
                DELETE FROM jobs
                WHERE job_id = $1 AND locked_by = $2
                ",
        )
        .bind(job.job_id)
        .bind(self.id.to_string())
        .execute(self.state.db())
        .await?;

        Ok(Ok(RunJobSuccess(job)))
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
    async fn fetch_next_job(&self) -> color_eyre::Result<Option<JobFromDB>> {
        let job = sqlx::query_as::<_, JobFromDB>(
            "
            UPDATE jobs
            SET LOCKED_BY = $1, LOCKED_AT = NOW()
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
        )
        .bind(self.id.to_string())
        .fetch_optional(self.state.db())
        .await?;

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
    async fn tick(&self) -> color_eyre::Result<()> {
        let job = self.fetch_next_job().await?;

        let Some(job) = job else {
            let duration = std::time::Duration::from_secs(5);
            tracing::debug!(worker.id =% self.id, ?duration, "No Job to Run, sleeping for requested duration");

            tokio::time::sleep(duration).await;

            return Ok(());
        };

        let result = self.run_next_job(job).await?;

        match result {
            Ok(RunJobSuccess(job)) => {
                tracing::info!(worker.id =% self.id, job_id =% job.job_id, "Job Ran");
            }
            Err(job_error) => {
                tracing::error!(
                    worker.id =% self.id,
                    job_id =% job_error.0.job_id,
                    error_msg =% job_error.1,
                    "Job Errored"
                );
            }
        }

        Ok(())
    }
}

pub async fn job_worker<AppState: AS>(
    app_state: AppState,
    registry: impl JobRegistry<AppState>,
) -> color_eyre::Result<()> {
    let worker = Worker::new(app_state, registry);

    loop {
        worker.tick().await?;
    }
}
