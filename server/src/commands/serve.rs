use crate::*;

pub(crate) async fn serve() -> Result<()> {
    let app_config = AppConfig::from_env()?;
    let twitch_config = TwitchConfig::from_env()?;
    let github_config = GithubConfig::from_env()?;
    let rss_config = RssConfig::from_env()?;
    let open_ai_config = OpenAiConfig::from_env()?;

    let database_url: String = std::env::var("DATABASE_URL").or_else(|_| -> Result<String> {
        let path = std::env::var("DATABASE_PATH");

        Ok(if let Ok(p) = &path {
            OpenOptions::new().write(true).create(true).open(p)?;

            format!("sqlite:{}", p)
        } else {
            "sqlite::memory:".to_string()
        })
    })?;

    let pool = SqlitePool::connect(&database_url).await?;

    let config = Config {
        twitch: twitch_config,
        db_pool: pool,
        github: github_config,
        app: app_config,
        rss: rss_config,
        open_ai: open_ai_config,
    };

    info!("About to run migrations (if any to apply)");
    migrate!("./migrations/").run(&config.db_pool).await?;

    let discord_bot = build_discord_bot(config.clone()).await?;

    // let http_and_cache = discord_bot.client().cache_and_http.clone();

    info!("Spawning Tasks");
    let discord_future = tokio::spawn(discord_bot.start());
    let axum_future = tokio::spawn(run_axum(config.clone()));
    info!("Tasks Spawned");

    let (discord_result, axum_result) = try_join!(discord_future, axum_future)?;

    discord_result?;
    axum_result?;

    info!("Main Returning");

    Ok(())
}
