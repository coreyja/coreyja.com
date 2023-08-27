use audio::run_audio_loop;

use miette::IntoDiagnostic;
pub use miette::Result;
use openai::OpenAiConfig;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing_common::setup_tracing;
use twitch::run_twitch_bot;

pub mod audio;
pub mod personality;
pub mod tts;
pub mod twitch;

async fn setup_db() -> Result<PgPool> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .into_diagnostic()?;

    sqlx::migrate!().run(&pool).await.into_diagnostic()?;

    Ok(pool)
}

#[derive(Debug, Clone)]
pub(crate) struct Config {
    db: PgPool,
    openai: OpenAiConfig,
    say: tokio::sync::mpsc::Sender<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing()?;

    let (say_sender, say_reciever) = tokio::sync::mpsc::channel::<String>(32);

    let pool = setup_db().await?;
    let openai = OpenAiConfig::from_env()?;
    let config = Config {
        db: pool,
        openai,
        say: say_sender.clone(),
    };

    let tasks = vec![
        tokio::task::spawn(tts::say_loop(say_reciever)),
        // tokio::task::spawn(run_twitch_bot(config.clone())),
        tokio::task::spawn(run_audio_loop(config.clone())),
    ];

    futures::future::try_join_all(tasks)
        .await
        .into_diagnostic()?;

    Ok(())
}
