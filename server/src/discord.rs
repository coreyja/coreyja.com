use std::sync::Arc;

use crate::*;

use miette::Context as _;
use poise::{
    futures_util::StreamExt,
    serenity_prelude::{EmojiId, ReactionType},
    Framework,
};

type Error = miette::Report;
type Context<'a> = poise::Context<'a, AppState, Error>;

#[poise::command(prefix_command, owners_only)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx)
        .await
        .into_diagnostic()?;
    Ok(())
}

#[poise::command(prefix_command, owners_only)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await.into_diagnostic()?;
    Ok(())
}

#[poise::command(prefix_command, ephemeral, owners_only)]
async fn whoami(ctx: Context<'_>) -> Result<(), Error> {
    let config = ctx.data();
    let author_id: i64 = ctx.author().id.0.try_into().into_diagnostic()?;

    let user = user_from_discord_user_id(author_id, &config.db_pool)
        .await
        .into_diagnostic()?;
    let user_id = user.id();

    async fn message_from_user(
        user: QueryOnRead<User>,
        config: &AppState,
        author_id: i64,
    ) -> Result<String> {
        let existing_twitch_link = user_twitch_link_from_user(&user, &config.db_pool)
            .await
            .into_diagnostic()?;
        let twitch_message = if let Some(existing_twitch_link) = existing_twitch_link {
            let twitch_login = existing_twitch_link.external_twitch_login;

            format!("You are linked as `{twitch_login}` on Twitch")
        } else {
            "You are not linked to Twitch. Use the buttons below to Authenticate with Twitch!"
                .to_string()
        };

        let existing_github_link = user_github_link_from_user(&user, &config.db_pool)
            .await
            .into_diagnostic()?;
        let github_message = if let Some(existing_github_link) = existing_github_link {
            let github_username = existing_github_link.external_github_username;

            format!("You are linked as `{github_username}` on Github")
        } else {
            "You are not linked to Github. Use the buttons below to authenticate with Github!"
                .to_string()
        };
        let message =
            format!("Your Discord ID is `{author_id}`\n\n{twitch_message}\n\n{github_message}");

        Ok(message)
    }

    let message = message_from_user(user, config, author_id).await?;
    let reply = ctx
        .send(|cr| {
            cr.content(message.clone()).components(|b| {
                b.create_action_row(|ar| {
                    ar.create_button(|b| b.label("Twitch").custom_id("link:twitch"))
                        .create_button(|b| b.label("Link Github").custom_id("link:github"))
                })
            })
        })
        .await
        .into_diagnostic()?;

    let mut interations = reply
        .message()
        .await
        .into_diagnostic()?
        .await_component_interactions(ctx.discord())
        .author_id(ctx.author().id)
        .timeout(Duration::from_secs(60))
        .build();

    while let Some(m) = interations.next().await {
        m.create_interaction_response(ctx.discord(), |ir| {
            ir.kind(serenity::InteractionResponseType::DeferredUpdateMessage)
        })
        .await
        .into_diagnostic()?;

        let pressed_button_id = &m.data.custom_id;

        let (label, url) = match pressed_button_id.as_str() {
            "link:twitch" => (
                "Link Twitch",
                generate_user_twitch_link(config, user_id).await?,
            ),
            "link:github" => {
                let url = generate_user_github_link(config, user_id).await?;

                ("Link Github", url)
            }
            _ => panic!("Unknown button pressed"),
        };

        ctx.send(|cr| {
            cr.content(label).components(|b| {
                b.create_action_row(|ar| {
                    ar.create_button(|b| {
                        let emoji: ReactionType = ReactionType::Custom {
                            animated: false,
                            id: EmojiId(1063871155062198282),
                            name: Some("github".to_owned()),
                        };
                        b.style(serenity::ButtonStyle::Link)
                            .label("Link Now!")
                            .emoji(emoji)
                            .url(url)
                    })
                })
            })
        })
        .await
        .into_diagnostic()?;

        let new_message = message_from_user(QueryOnRead::Id(user_id), config, author_id).await?;
        if new_message != message {
            reply
                .edit(ctx, |b| b.content(new_message))
                .await
                .into_diagnostic()?;
        }
    }

    reply.delete(ctx).await.into_diagnostic()?;

    Ok(())
}

pub(crate) async fn build_discord_bot(
    config: AppState,
) -> Result<Arc<Framework<AppState, miette::Report>>> {
    let framework = poise::Framework::builder()
        .initialize_owners(true)
        .options(poise::FrameworkOptions {
            commands: vec![register(), ping(), whoami()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .token(
            std::env::var("DISCORD_TOKEN")
                .into_diagnostic()
                .wrap_err("missing DISCORD_TOKEN")?,
        )
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(config) }));

    framework.build().await.into_diagnostic()
}
