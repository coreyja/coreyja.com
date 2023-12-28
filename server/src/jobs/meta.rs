use miette::{Diagnostic, IntoDiagnostic, Result};
use thiserror::Error;

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
    JobRan(JobId),
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

#[tracing::instrument(
    name = "worker.run_next_job",
    skip(app_state),
    fields(
        worker.id = worker_id,
    ),
    ret,
    err,
)]
pub(crate) async fn run_next_job(
    app_state: crate::AppState,
    worker_id: &str,
) -> Result<RunJobResult> {
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
        worker_id,
        now,
    )
    .fetch_optional(&app_state.db)
    .await
    .into_diagnostic()?;

    let Some(job) = job else {
        return Ok(Ok(RunJobSuccess::NoRunnableJobInQueue));
    };

    let job_id = JobId(job.job_id);
    let job_result = run_job(&job, &app_state, worker_id).await;

    if let Err(e) = job_result {
        sqlx::query!(
            "
          UPDATE jobs
          SET locked_by = NULL, locked_at = NULL, run_at = now() + interval '60 seconds'
          WHERE job_id = $1 AND locked_by = $2
              ",
            job.job_id,
            worker_id
        )
        .execute(&app_state.db)
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
        worker_id
    )
    .execute(&app_state.db)
    .await
    .into_diagnostic()?;

    Ok(Ok(RunJobSuccess::JobRan(job_id)))
}

#[tracing::instrument(
    name = "worker.run_job",
    skip(app_state, job, worker_id),
    fields(
        job.id = %job.job_id,
        job.name = job.name,
        job.priority = job.priority,
        job.run_at = %job.run_at,
        job.created_at = %job.created_at,
        job.context = job.context,
        worker.id = worker_id,
    )
    err,
)]
async fn run_job(
    job: &JobFromDB,
    app_state: &crate::state::AppState,
    worker_id: &str,
) -> Result<()> {
    let payload = job.payload.clone();

    match job.name.as_str() {
        "RefreshSponsors" => RefreshSponsors::run_from_value(payload, app_state.clone()).await,
        "RefreshVideos" => {
            super::youtube_videos::RefreshVideos::run_from_value(payload, app_state.clone()).await
        }
        _ => Err(miette::miette!("Unknown job type: {}", job.name)),
    }
}

#[derive(Debug, Diagnostic, Error)]
#[error("JobError(id:${1}: ${0}")]
pub(crate) struct JobError(JobId, miette::Report);

pub(crate) async fn job_worker(app_state: crate::AppState) -> Result<()> {
    let worker_id = uuid::Uuid::new_v4().to_string();
    loop {
        tracing::info!(%worker_id, "About to attempt to pickup and run next job");
        let result = run_next_job(app_state.clone(), &worker_id).await?;

        match result {
            Ok(RunJobSuccess::JobRan(job_id)) => {
                tracing::info!(%worker_id, job_id =% job_id.0, "Job Ran");
            }
            Ok(RunJobSuccess::NoRunnableJobInQueue) => {
                let duration = std::time::Duration::from_secs(5);
                tracing::debug!(%worker_id, ?duration, "No Job to Run, sleeping for requested duration");

                tokio::time::sleep(duration).await;
            }
            Err(job_error) => {
                sentry::capture_error(&job_error);
                tracing::error!(
                    %worker_id,
                    job_id =% job_error.0.0,
                    error_msg =% job_error.1,
                    "Job Errored"
                );
            }
        }
    }
}
