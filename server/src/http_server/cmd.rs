use miette::{IntoDiagnostic, Result};
use tracing::info;

use crate::{cron::run_cron, http_server::run_axum, jobs::meta::job_worker, AppState};

pub(crate) async fn serve() -> Result<()> {
    let app_state = AppState::from_env().await?;

    info!("Spawning Tasks");
    let axum_future = tokio::spawn(run_axum(app_state.clone()));
    let worker_future = tokio::spawn(job_worker(app_state.clone()));
    let cron_future = tokio::spawn(run_cron(app_state.clone()));
    info!("Tasks Spawned");

    axum_future.await.into_diagnostic()??;
    worker_future.await.into_diagnostic()??;
    cron_future.await.into_diagnostic()??;

    info!("Main Returning");

    Ok(())
}
