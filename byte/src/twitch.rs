use db::twitch_chatters::{
    get_or_create_twitch_chatter, preferred_twitch_name, update_twitch_chatter_nickname,
};
use futures::StreamExt;
use irc::{
    client::{prelude::Config, Client},
    proto::{Capability, Prefix},
};
use miette::{IntoDiagnostic, Result};
use openai::chat::{complete_chat, ChatMessage, ChatRole};

use crate::personality::{base, respond_to_twitch_chat_prompt};

pub(crate) async fn run_twitch_bot(config: super::Config) -> Result<()> {
    let irc_config = Config {
        nickname: Some("coreyja_bot".to_owned()),
        password: Some(format!(
            "oauth:{}",
            std::env::var("TWITCH_BOT_ACCESS_TOKEN").into_diagnostic()?
        )),
        server: Some("irc.chat.twitch.tv".to_owned()),
        channels: vec!["#coreyja".to_owned()],
        ..Config::default()
    };

    let mut client = Client::from_config(irc_config).await.into_diagnostic()?;
    client.identify().into_diagnostic()?;

    let mut stream = client.stream().into_diagnostic()?;

    client
        .send_cap_req(&[Capability::Custom("twitch.tv/membership")])
        .into_diagnostic()?;

    while let Some(message) = stream.next().await.transpose().into_diagnostic()? {
        match &message.command {
            irc::proto::Command::PRIVMSG(_target, msg) => {
                if msg.starts_with("!byte") {
                    let chat_msg = msg.strip_prefix("!byte").unwrap();
                    if let Some(Prefix::Nickname(nickname, _username, _hostname)) = &message.prefix
                    {
                        let preferred_name = preferred_twitch_name(&config.db, nickname).await?;
                        let messages = vec![
                            base(),
                            respond_to_twitch_chat_prompt(),
                            ChatMessage {
                                role: ChatRole::User,
                                content: format!("{}: {}", preferred_name, chat_msg),
                            },
                        ];
                        let resp = complete_chat(&config.openai, "gpt-3.5-turbo", messages).await?;

                        config.say.send(resp.content).await.into_diagnostic()?;
                    }
                } else if msg.starts_with("!nickname") {
                    let chat_msg = msg.strip_prefix("!nickname").unwrap();

                    if let Some(Prefix::Nickname(nickname, _username, _hostname)) = &message.prefix
                    {
                        if chat_msg.is_empty() {
                            let user = get_or_create_twitch_chatter(&config.db, nickname).await?;

                            let msg = match user.preferred_name {
                                Some(name) => {
                                    format!("Your current nickname is {name}")
                                }
                                None => "You don't have a nickname set".to_string(),
                            };

                            client.send_privmsg("#coreyja", msg).into_diagnostic()?;
                        } else {
                            let user = get_or_create_twitch_chatter(&config.db, nickname).await?;
                            let old_nickname = &user.preferred_name;

                            let new_nickname = chat_msg.trim();

                            update_twitch_chatter_nickname(&config.db, &user, new_nickname).await?;

                            let msg = format!(
                                "Your nickname has been updated from {} to {}",
                                old_nickname.as_ref().unwrap_or(&user.twitch_username),
                                new_nickname
                            );
                            client.send_privmsg("#coreyja", msg).into_diagnostic()?;
                        }
                    }
                }
            }
            irc::proto::Command::JOIN(_, _, _) => {
                if let Some(Prefix::Nickname(nick, _, _)) = message.prefix {
                    if nick == client.current_nickname() {
                        continue;
                    }

                    let preferred_name = preferred_twitch_name(&config.db, &nick).await?;

                    let prompt = format!("{} has joined the channel", preferred_name);
                    println!("{}", prompt);

                    let resp = complete_chat(
                        &config.openai,
                        "gpt-3.5-turbo",
                        vec![
                            base(),
                            ChatMessage {
                                role: ChatRole::User,
                                content: prompt,
                            },
                        ],
                    )
                    .await?;
                    config.say.send(resp.content).await.into_diagnostic()?;
                };
            }
            _ => {
                println!("Unhandled message: {:?}", message);
            }
        }
    }

    Ok(())
}
