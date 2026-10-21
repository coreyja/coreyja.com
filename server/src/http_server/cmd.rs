use std::time::Duration;

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::Host;
use cja::server::session::Session;
use cja::tasks::{ShutdownBudget, Supervisor};
use cja::Result;
use cja::{
    jobs::{worker::job_worker, JobWorkerConfig},
    server::run_server_until,
};
use serde_json::Map;
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

    let app_state = AppState::from_env(discord.client.clone()).await?;

    *discord.app_state_holder.lock().unwrap() = Some(app_state.clone());
    let job_registry = Jobs;

    // Build the cron registry once so it can seed the Eyes boot manifest below
    // and then drive the cron worker (when enabled).
    let cron_registry = crate::cron::cron_registry();

    // Emit this app's shape (job types, cron schedules, build version) to Eyes
    // at boot; fire-and-forget and a no-op unless EYES_ORG_ID/EYES_APP_ID are
    // set. Reflects the full deployed shape even if JOBS_DISABLED/CRON_DISABLED
    // turn the workers off for this process.
    cja::eyes_manifest::send_boot_manifest::<Jobs, AppState>(
        Some(env!("CARGO_PKG_VERSION")),
        option_env!("VERGEN_GIT_SHA"),
        Some(&cron_registry),
    );

    // Registers SIGTERM and SIGINT (Fly sends SIGINT, see `kill_signal` in
    // fly.toml) and stops every task within the budget. The default budget is
    // 2s of job drain plus 2s of exit grace, which fits the 5s `kill_timeout`.
    let mut supervisor = Supervisor::new(ShutdownBudget::from_env())?;
    let shutdown = supervisor.shutdown_token();

    info!("Spawning Tasks");
    supervisor.spawn(
        "server",
        run_server_until(
            routes::make_router()
                .layer(axum::middleware::from_fn_with_state(
                    app_state.clone(),
                    pageview_middleware,
                ))
                .with_state(app_state.clone()),
            shutdown.clone().cancelled_owned(),
        ),
    );

    if std::env::var("JOBS_DISABLED").unwrap_or_else(|_| "false".to_string()) == "false" {
        info!("Jobs Enabled");
        supervisor.spawn(
            "jobs",
            job_worker(
                app_state.clone(),
                job_registry,
                Duration::from_millis(500),
                cja::jobs::DEFAULT_MAX_RETRIES,
                shutdown.clone(),
                JobWorkerConfig {
                    shutdown_drain_timeout: supervisor.budget().job_drain,
                    ..JobWorkerConfig::default()
                },
            ),
        );
    } else {
        info!("Jobs Disabled");
    }

    if std::env::var("CRON_DISABLED").unwrap_or_else(|_| "false".to_string()) == "false" {
        info!("Cron Enabled");
        supervisor.spawn(
            "cron",
            run_cron(app_state.clone(), cron_registry, shutdown.clone()),
        );
    } else {
        info!("Cron Disabled");
    }

    if std::env::var("DISCORD_BOT_DISABLED").unwrap_or_else(|_| "false".to_string()) == "false" {
        info!("Discord Bot Enabled");
        supervisor.spawn("discord", run_discord_bot(discord.bot, shutdown));
    } else {
        info!("Discord Bot Disabled");
    }

    info!("Tasks Spawned");

    let result = supervisor.run().await;

    info!("Main Returning");

    result
}

/// The supervisor treats a task that ends by itself as fatal and stops the
/// process. The site should stay up without the bot (as it did before the
/// supervisor), so a bot failure is logged and the task then idles until
/// shutdown instead of returning.
async fn run_discord_bot(
    bot: crate::discord::DiscordBot,
    shutdown: cja::jobs::CancellationToken,
) -> Result<()> {
    if let Err(error) = bot.run(shutdown.clone()).await {
        tracing::error!(error = %format!("{error:#}"), "Discord bot stopped; continuing without it");
    }
    shutdown.cancelled().await;

    Ok(())
}
