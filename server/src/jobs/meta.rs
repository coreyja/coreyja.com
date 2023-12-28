use miette::{Diagnostic, ErrReport, IntoDiagnostic, Result};
use thiserror::Error;

use super::{sponsors::RefreshSponsors, Job};

#[derive(Debug, Clone)]
pub struct JobId(uuid::Uuid);

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.to_string().fmt(f)
    }
}

pub type RunJobResult = Result<RunJobSuccess, JobError>;
pub enum RunJobSuccess {
    JobRan(JobId),
    NoRunnableJobInQueue,
}

pub(crate) async fn run_next_job(
    app_state: crate::AppState,
    worker_id: &str,
) -> Result<RunJobResult> {
    let now = chrono::Utc::now();
    let job = sqlx::query!(
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
    let job_result = match job.name.as_str() {
        "RefreshSponsors" => RefreshSponsors::run_from_value(job.payload, app_state.clone()).await,
        "RefreshVideos" => {
            super::youtube_videos::RefreshVideos::run_from_value(job.payload, app_state.clone())
                .await
        }
        _ => return Err(miette::miette!("Unknown job type: {}", job.name)),
    };

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

#[derive(Debug, Diagnostic, Error)]
#[error("JobError(id:${1}: ${0}")]
pub(crate) struct JobError(JobId, miette::Report);

pub(crate) async fn job_worker(app_state: crate::AppState) -> Result<()> {
    loop {
        let worker_id = uuid::Uuid::new_v4().to_string();
        tracing::info!(%worker_id, "About to pickup next job");
        let result = run_next_job(app_state.clone(), &worker_id).await?;

        match result {
            Ok(RunJobSuccess::JobRan(job_id)) => {
                tracing::info!(%worker_id, job_id =% job_id.0, "Job Ran");
            }
            Ok(RunJobSuccess::NoRunnableJobInQueue) => {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
            Err(job_error) => {
                sentry::capture_error(&job_error);
                tracing::info!(
                    %worker_id,
                    job_id =% job_error.0.0,
                    error =% job_error.1,
                    "Job Errored"
                );
            }
        }
    }
}
