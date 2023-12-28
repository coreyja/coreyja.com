use miette::{IntoDiagnostic, Result};
use tokio::task::JoinError;
use tracing::info;

use crate::{cron::run_cron, http_server::run_axum, jobs::worker::job_worker, AppState};

pub(crate) async fn serve() -> Result<()> {
    let app_state = AppState::from_env().await?;

    info!("Spawning Tasks");
    let futures = vec![
        tokio::spawn(run_axum(app_state.clone())),
        tokio::spawn(job_worker(app_state.clone())),
        tokio::spawn(run_cron(app_state.clone())),
    ];
    info!("Tasks Spawned");

    let results = futures::future::join_all(futures).await;
    let results: Result<Vec<Result<()>>, JoinError> = results.into_iter().collect();
    results
        .into_diagnostic()?
        .into_iter()
        .collect::<Result<Vec<()>>>()?;

    info!("Main Returning");

    Ok(())
}
