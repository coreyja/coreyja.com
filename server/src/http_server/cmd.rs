use axum::extract::{Host, Request, State};
use axum::middleware::Next;
use axum::response::Response;
use cja::Result;
use cja::{jobs::worker::job_worker, server::run_server};
use serde_json::Map;
use tokio::task::JoinError;
use tracing::info;
use url::Url;

use crate::tracking;
use crate::{cron::run_cron, http_server::routes, jobs::Jobs, AppState};

use super::current_user::CurrentUser;

const IGNORED_PATH_PREFIXES: &[&str] = &["/static", "/styles"];

async fn pageview_middleware(
    State(state): State<AppState>,
    current_user: Option<CurrentUser>,
    Host(hostname): Host,
    request: Request,
    next: Next,
) -> Response {
    let uri = request.uri();

    let path = uri.path();
    let parsed_path = std::path::Path::new(&path);
    let extension = parsed_path.extension();

    if IGNORED_PATH_PREFIXES
        .iter()
        .any(|prefix| path.starts_with(prefix))
    {
        tracing::debug!(path =? path, "Ignoring pageview event for ignored path prefix");

        return next.run(request).await;
    }

    if let Some(ext) = extension {
        let ext = ext.to_string_lossy();
        tracing::debug!(path =? path, extension =% ext, "Ignoring pageview event for path with extension");

        return next.run(request).await;
    }

    let mut props = Map::new();
    props.insert("$current_url".to_string(), uri.to_string().into());
    props.insert("$host".to_string(), hostname.into());

    if tracking::posthog::capture_event(
        &state,
        "$pageview",
        current_user.map(|u| u.user.user_id),
        Some(props),
    )
    .await
    .is_err()
    {
        tracing::error!("Failed to capture pageview event");
    }

    next.run(request).await
}

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
            routes::make_router(syntax_css)
                .layer(axum::middleware::from_fn_with_state(
                    app_state.clone(),
                    pageview_middleware,
                ))
                .with_state(app_state.clone()),
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
