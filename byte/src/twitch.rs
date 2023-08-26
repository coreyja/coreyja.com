use futures::StreamExt;
use irc::{
    client::{prelude::Config, Client},
    proto::Prefix,
};
use miette::{IntoDiagnostic, Result};
use openai::chat::{complete_chat, ChatMessage, ChatRole};
use tokio::process::Command;

use crate::{
    personality::{base, respond_to_twitch_chat_prompt},
    tts::say,
};

pub async fn run_twitch_bot(say_sender: tokio::sync::mpsc::Sender<String>) -> Result<()> {
    let openai_config = openai::OpenAiConfig::from_env()?;
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
        if let irc::proto::Command::PRIVMSG(_target, msg) = &message.command {
            let Some(chat_msg) = msg.strip_prefix("!byte") else {
                continue;
            };

            if let Some(Prefix::Nickname(nickname, _username, _hostname)) = &message.prefix {
                let messages = vec![
                    base(),
                    respond_to_twitch_chat_prompt(),
                    ChatMessage {
                        role: ChatRole::User,
                        content: format!("{}: {}", nickname, chat_msg),
                    },
                ];
                let resp = complete_chat(&openai_config, "gpt-3.5-turbo", messages).await?;

                say_sender.send(resp.content).await.into_diagnostic()?;
            }
        };
    }

    Ok(())
}
