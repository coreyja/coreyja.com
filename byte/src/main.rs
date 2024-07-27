use audio::run_audio_loop;

pub use color_eyre::Result;
use db::{setup_db_pool, PgPool};
use openai::OpenAiConfig;
use tracing_common::setup_tracing;
use twitch::run_twitch_bot;

pub mod audio;
pub mod personality;
pub mod tts;
pub mod twitch;

#[derive(Debug, Clone)]
pub(crate) struct Config {
    db: PgPool,
    openai: OpenAiConfig,
    say: tokio::sync::mpsc::Sender<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing("byte").unwrap();

    let (say_sender, say_reciever) = tokio::sync::mpsc::channel::<String>(32);

    let pool = setup_db_pool().await?;
    let openai = OpenAiConfig::from_env()?;
    let config = Config {
        db: pool,
        openai,
        say: say_sender.clone(),
    };

    let tasks = vec![
        tokio::task::spawn(tts::say_loop(say_reciever)),
        tokio::task::spawn(run_twitch_bot(config.clone())),
        tokio::task::spawn(run_audio_loop(config.clone())),
    ];

    futures::future::try_join_all(tasks).await?;

    Ok(())
}
