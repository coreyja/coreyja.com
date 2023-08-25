use futures::StreamExt;
use irc::client::{prelude::Config, Client};
use miette::{IntoDiagnostic, Result};

pub async fn run_twitch_bot() -> Result<()> {
    let config = Config {
        nickname: Some("coreyja_bot".to_owned()),
        password: Some(format!(
            "oauth:{}",
            std::env::var("TWITCH_BOT_ACCESS_TOKEN").into_diagnostic()?
        )),
        server: Some("irc.chat.twitch.tv".to_owned()),
        channels: vec!["#coreyja".to_owned()],
        ..Config::default()
    };

    let mut client = Client::from_config(config).await.into_diagnostic()?;
    client.identify().into_diagnostic()?;

    let mut stream = client.stream().into_diagnostic()?;

    while let Some(message) = stream.next().await.transpose().into_diagnostic()? {
        print!("{}", message);
    }

    Ok(())
}
