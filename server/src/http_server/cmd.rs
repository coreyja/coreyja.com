use base64::Engine;
use db::setup_db_pool;

use crate::{http_server::pages::blog::md::SyntaxHighlightingContext, jobs::meta::job_worker, *};

pub(crate) async fn serve() -> Result<()> {
    let blog_posts = BlogPosts::from_static_dir()?;
    let blog_posts = Arc::new(blog_posts);

    let til_posts = TilPosts::from_static_dir()?;
    let til_posts = Arc::new(til_posts);

    let streams = PastStreams::from_static_dir()?;
    let streams = Arc::new(streams);

    let projects = Projects::from_static_dir()?;
    let projects = Arc::new(projects);

    let cookie_key = std::env::var("COOKIE_KEY");
    let cookie_key = if let Ok(cookie_key) = cookie_key {
        let cookie_key = base64::engine::general_purpose::STANDARD
            .decode(cookie_key.as_bytes())
            .into_diagnostic()?;

        Key::derive_from(&cookie_key)
    } else {
        info!("Generating new cookie key");
        Key::generate()
    };
    let cookie_key = DebugIgnore(cookie_key);

    let app_state = AppState {
        twitch: TwitchConfig::from_env()?,
        github: GithubConfig::from_env()?,
        app: AppConfig::from_env()?,
        open_ai: OpenAiConfig::from_env()?,
        markdown_to_html_context: SyntaxHighlightingContext::default(),
        versions: VersionInfo::from_env(),
        blog_posts,
        til_posts,
        streams,
        projects,
        db: setup_db_pool().await?,
        cookie_key,
        encrypt_config: encrypt::Config::from_env()?,
    };

    info!("Spawning Tasks");
    let axum_future = tokio::spawn(run_axum(app_state.clone()));
    let worker_future = tokio::spawn(job_worker(app_state.clone()));
    let cron_future = tokio::spawn(cron::run_cron(app_state.clone()));
    info!("Tasks Spawned");

    axum_future.await.into_diagnostic()??;
    worker_future.await.into_diagnostic()??;
    cron_future.await.into_diagnostic()??;

    info!("Main Returning");

    Ok(())
}
