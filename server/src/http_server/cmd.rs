use cja::Result;
use cja::{jobs::worker::job_worker, server::run_server};
use tokio::task::JoinError;
use tracing::info;

use crate::{
    cron::{self, run_cron},
    http_server::routes,
    jobs::Jobs,
    AppState,
};

pub(crate) async fn serve() -> Result<()> {
    let app_state = AppState::from_env().await?;
    let job_registry = Jobs;

    let syntax_css = syntect::html::css_for_theme_with_class_style(
        &app_state.syntax_highlighting_context.theme,
        syntect::html::ClassStyle::Spaced,
    )?;

    info!("Spawning Tasks");
    let mut futures: Vec<tokio::task::JoinHandle<Result<()>>> = vec![
        tokio::spawn(run_server(
            routes::make_router(syntax_css).with_state(app_state.clone()),
        )),
        tokio::spawn(job_worker(app_state.clone(), job_registry)),
    ];

    if std::env::var("CRON_DISABLED").unwrap_or_else(|_| "false".to_string()) == "false" {
        info!("Cron Enabled");
        futures.push(tokio::spawn(run_cron(app_state.clone())));
    } else {
        info!("Cron Disabled");
    }

    info!("Tasks Spawned");

    let results = futures::future::join_all(futures).await;
    let results: Result<Vec<Result<()>>, JoinError> = results.into_iter().collect();
    results?.into_iter().collect::<Result<Vec<()>>>()?;

    info!("Main Returning");

    Ok(())
}
