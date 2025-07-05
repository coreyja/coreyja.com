use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::Host;
use cja::server::session::Session;
use cja::Result;
use cja::{jobs::worker::job_worker, server::run_server};
use serde_json::Map;
use tokio::task::JoinError;
use tracing::info;

use crate::http_server::auth::session::DBSession;
use crate::tracking;
use crate::{cron::run_cron, http_server::routes, jobs::Jobs, AppState};

const IGNORED_PATH_PREFIXES: &[&str] = &["/static", "/styles"];

#[axum_macros::debug_middleware(state = AppState)]
async fn pageview_middleware(
    State(state): State<AppState>,
    Host(hostname): Host,
    Session(session): Session<DBSession>,
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

    let fly_region = request
        .headers()
        .get("fly-region")
        .and_then(|h| h.to_str().ok());
    if let Some(fly_region) = fly_region {
        props.insert("fly-region".to_string(), fly_region.into());
    }

    let mut user_id = session.user_id.map(|u| u.to_string());

    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok());

    if let Some(user_agent) = user_agent {
        if user_agent.contains("DigitalOcean Uptime Probe") {
            user_id = Some("service:digitalocean-uptime-probe".to_string());
        }

        props.insert("$user_agent".to_string(), user_agent.into());
    }

    let referrer = request
        .headers()
        .get("referer")
        .and_then(|h| h.to_str().ok());
    if let Some(referrer) = referrer {
        props.insert("$referrer".to_string(), referrer.into());

        let parsed_referer = url::Url::parse(referrer);
        if let Ok(parsed_referer) = parsed_referer {
            let host = parsed_referer.host_str();

            if let Some(host) = host {
                props.insert("$referring_domain".to_string(), host.into());
            } else {
                props.insert("$referring_domain".to_string(), "$direct".into());
            }
        } else {
            props.insert("$referring_domain".to_string(), "$direct".into());
        }
    } else {
        props.insert("$referrer".to_string(), "$direct".into());
        props.insert("$referring_domain".to_string(), "$direct".into());
    }

    if tracking::posthog::capture_event(&state, "$pageview", user_id, Some(props))
        .await
        .is_err()
    {
        tracing::error!("Failed to capture pageview event");
    }

    next.run(request).await
}

pub(crate) async fn serve() -> Result<()> {
    let discord = crate::discord::setup().await?;

    let app_state = AppState::from_env(discord.client).await?;
    let job_registry = Jobs;

    let syntax_css = syntect::html::css_for_theme_with_class_style(
        &app_state.syntax_highlighting_context.theme,
        syntect::html::ClassStyle::Spaced,
    )?;

    info!("Spawning Tasks");
    let mut futures: Vec<tokio::task::JoinHandle<Result<()>>> = vec![tokio::spawn(run_server(
        routes::make_router(syntax_css)
            .layer(axum::middleware::from_fn_with_state(
                app_state.clone(),
                pageview_middleware,
            ))
            .with_state(app_state.clone()),
    ))];

    if std::env::var("JOBS_DISABLED").unwrap_or_else(|_| "false".to_string()) == "false" {
        info!("Jobs Enabled");
        futures.push(tokio::spawn(job_worker(app_state.clone(), job_registry)));
    } else {
        info!("Jobs Disabled");
    }

    if std::env::var("CRON_DISABLED").unwrap_or_else(|_| "false".to_string()) == "false" {
        info!("Cron Enabled");
        futures.push(tokio::spawn(run_cron(app_state.clone())));
    } else {
        info!("Cron Disabled");
    }

    if std::env::var("DISCORD_BOT_DISABLED").unwrap_or_else(|_| "false".to_string()) == "false" {
        info!("Discord Bot Enabled");
        futures.push(tokio::spawn(discord.bot.run()));
    } else {
        info!("Discord Bot Disabled");
    }
    
    if std::env::var("BLUESKY_FIREHOSE_DISABLED").unwrap_or_else(|_| "false".to_string()) == "false" {
        info!("Bluesky Firehose Enabled");
        futures.push(tokio::spawn(crate::bsky::firehose::start_bluesky_firehose(app_state.clone())));
    } else {
        info!("Bluesky Firehose Disabled");
    }

    info!("Tasks Spawned");

    let results = futures::future::join_all(futures).await;
    let results: Result<Vec<Result<()>>, JoinError> = results.into_iter().collect();
    results?.into_iter().collect::<Result<Vec<()>>>()?;

    info!("Main Returning");

    Ok(())
}
