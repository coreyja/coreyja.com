use cja::{jobs::worker::job_worker, server::run_server};
use miette::{IntoDiagnostic, Result};
use tokio::task::JoinError;
use tracing::info;

use crate::{cron::run_cron, http_server::routes, jobs::Jobs, AppState};

pub(crate) async fn serve() -> Result<()> {
    let app_state = AppState::from_env().await?;

    let job_registry = Jobs;

    let syntax_css = syntect::html::css_for_theme_with_class_style(
        &app_state.markdown_to_html_context.theme,
        syntect::html::ClassStyle::Spaced,
    )
    .into_diagnostic()?;

    info!("Spawning Tasks");
    let futures = vec![
        tokio::spawn(run_server(
            routes::make_router(syntax_css).with_state(app_state.clone()),
        )),
        tokio::spawn(job_worker(app_state.clone(), job_registry)),
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
