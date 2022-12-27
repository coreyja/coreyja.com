use crate::*;

use color_eyre::eyre::WrapErr;
use uuid::Uuid;

type Error = color_eyre::Report;
type Context<'a> = poise::Context<'a, Config, Error>;

/// Displays your or another user's account creation date
#[poise::command(prefix_command, slash_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(context_menu_command = "Age")]
async fn user_age(ctx: Context<'_>, u: serenity::User) -> Result<(), Error> {
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(context_menu_command = "Author Age")]
async fn author_age(ctx: Context<'_>, msg: serenity::Message) -> Result<(), Error> {
    let u = msg.author;
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(prefix_command, owners_only)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[poise::command(prefix_command, owners_only)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    warn!("Pong!");
    ctx.say("Pong!").await?;
    Ok(())
}

#[poise::command(slash_command, ephemeral)]
async fn me(ctx: Context<'_>) -> Result<(), Error> {
    let config = ctx.data();
    let author_id: i64 = ctx.author().id.0.try_into()?;

    let user = user_from_discord_user_id(author_id, &config.db_pool).await?;
    let user_id = user.id();

    let existing_twitch_link = user_twitch_link_from_user(&user, &config.db_pool).await?;
    let twitch_message = if let Some(existing_twitch_link) = existing_twitch_link {
        let twitch_login = existing_twitch_link.external_twitch_login;

        format!("You are linked as `{twitch_login}` on Twitch")
    } else {
        let state = Uuid::new_v4().to_string();
        sqlx::query!(
            "INSERT INTO UserTwitchLinkStates (user_id, state) VALUES (?, ?)",
            user_id,
            state,
        )
        .execute(&config.db_pool)
        .await?;

        let url = generate_user_twitch_link(config, &state)?;

        format!("You are not linked to Twitch. Click here to login with Twitch: {url}")
    };

    let existing_github_link = user_github_link_from_user(&user, &config.db_pool).await?;
    let github_message = if let Some(existing_github_link) = existing_github_link {
        let github_username = existing_github_link.external_github_username;

        format!("You are linked as `{github_username}` on Github")
    } else {
        let state = Uuid::new_v4().to_string();
        sqlx::query!(
            "INSERT INTO UserGithubLinkStates (user_id, state) VALUES (?, ?)",
            user_id,
            state,
        )
        .execute(&config.db_pool)
        .await?;

        let url = generate_user_github_link(config, &state)?;

        format!("You are not linked to Github. Click here to login with Github: {url}")
    };

    ctx.say(format!(
        "Your Discord ID is `{author_id}`\n\n{twitch_message}\n\n{github_message}"
    ))
    .await?;

    Ok(())
}

pub(crate) async fn run_discord_bot(config: Config) -> Result<()> {
    let framework = poise::Framework::builder()
        .initialize_owners(true)
        .options(poise::FrameworkOptions {
            commands: vec![age(), register(), ping(), user_age(), author_age(), me()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").wrap_err("missing DISCORD_TOKEN")?)
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(config) }));

    framework.run().await?;

    Ok(())
}
