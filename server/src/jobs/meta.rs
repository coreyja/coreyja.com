use miette::{IntoDiagnostic, Result};

use super::{sponsors::RefreshSponsors, Job};

pub struct JobId(uuid::Uuid);

pub enum RunJobResult {
    JobRan(JobId),
    JobErrored(JobId),
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
        SET LOCKED_BY = $2, LOCKED_AT = $3
        WHERE job_id = (
            SELECT job_id
            FROM jobs
            WHERE run_at <= $1 AND locked_by IS NULL
            ORDER BY priority DESC, created_at ASC
            LIMIT 1
            FOR UPDATE SKIP LOCKED
        )
        RETURNING job_id, name, payload, priority, run_at, created_at, context
        ",
        now,
        worker_id,
        now,
    )
    .fetch_optional(&app_state.db)
    .await
    .into_diagnostic()?;

    let Some(job) = job else {
        return Ok(RunJobResult::NoRunnableJobInQueue);
    };

    let job_result = match job.name.as_str() {
        "RefreshSponsors" => {
            let job: RefreshSponsors = serde_json::from_value(job.payload).into_diagnostic()?;
            job.run(app_state.clone()).await
        }
        _ => return Err(miette::miette!("Unknown job type: {}", job.name)),
    };

    if job_result.is_err() {
        sqlx::query!(
            "
          UPDATE jobs
          SET locked_by = NULL, locked_at = NULL, run_at = $1
          WHERE job_id = $2 AND locked_by = $3
              ",
            now + chrono::Duration::seconds(60),
            job.job_id,
            worker_id
        )
        .execute(&app_state.db)
        .await
        .into_diagnostic()?;

        return Ok(RunJobResult::JobErrored(JobId(job.job_id)));
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

    Ok(RunJobResult::JobRan(JobId(job.job_id)))
}

pub(crate) async fn job_worker(app_state: crate::AppState) -> Result<()> {
    loop {
        let worker_id = uuid::Uuid::new_v4().to_string();
        tracing::info!(%worker_id, "About to pickup next job");
        let result = run_next_job(app_state.clone(), &worker_id).await?;

        match result {
            RunJobResult::JobRan(job_id) => {
                tracing::info!(%worker_id, job_id =% job_id.0, "Job Ran");
            }
            RunJobResult::JobErrored(job_id) => {
                tracing::info!(%worker_id, job_id =% job_id.0, "Job Errored");
            }
            RunJobResult::NoRunnableJobInQueue => {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}
