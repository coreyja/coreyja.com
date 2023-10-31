use crate::{http_server::pages::blog::md::SyntaxHighlightingContext, *};

pub(crate) async fn serve() -> Result<()> {
    let app_config = AppConfig::from_env()?;
    let twitch_config = TwitchConfig::from_env()?;
    let github_config = GithubConfig::from_env()?;
    let open_ai_config = OpenAiConfig::from_env()?;
    let markdown_to_html_context = SyntaxHighlightingContext::default();

    let blog_posts = BlogPosts::from_static_dir()?;
    let blog_posts = Arc::new(blog_posts);

    let til_posts = TilPosts::from_static_dir()?;
    let til_posts = Arc::new(til_posts);

    let streams = PastStreams::from_static_dir()?;
    let streams = Arc::new(streams);

    let app_state = AppState {
        twitch: twitch_config,
        github: github_config,
        app: app_config,
        open_ai: open_ai_config,
        markdown_to_html_context,
        blog_posts,
        til_posts,
        streams,
    };

    info!("Spawning Tasks");
    let axum_future = tokio::spawn(run_axum(app_state.clone()));
    info!("Tasks Spawned");

    // let (discord_result, axum_result) = try_join!(discord_future, axum_future).into_diagnostic()?;

    // discord_result.into_diagnostic()?;
    axum_future.await.into_diagnostic()??;

    info!("Main Returning");

    Ok(())
}
