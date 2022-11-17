use uuid::Uuid;

use crate::*;

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

#[poise::command(prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    warn!("Pong!");
    ctx.say("Pong!").await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, ephemeral)]
async fn twitch(ctx: Context<'_>) -> Result<(), Error> {
    let config = ctx.data();
    let author_id: i64 = ctx.author().id.0.try_into()?;

    let existing_twitch_link = sqlx::query!(
        "SELECT * FROM DiscordTwitchLinks WHERE discord_user_id = $1",
        author_id
    )
    .fetch_optional(&config.db_pool)
    .await?;

    if let Some(existing_twitch_link) = existing_twitch_link {
        let twitch_login = existing_twitch_link.twitch_login;
        ctx.say(format!(
            "You are already linked as `{twitch_login}` on Twitch"
        ))
        .await?;
    } else {
        let state = Uuid::new_v4().to_string();
        sqlx::query!(
            "INSERT INTO TwitchLinkStates (discord_user_id, state) VALUES (?, ?)",
            author_id,
            state,
        )
        .execute(&config.db_pool)
        .await?;

        let url = generate_user_twitch_link(&config.twitch, &state)?;

        ctx.say(format!("Twitch Verify: {url}")).await?;
    }

    Ok(())
}

pub(crate) async fn run_discord_bot(config: Config) -> Result<()> {
    let framework = poise::Framework::builder()
        .initialize_owners(true)
        .options(poise::FrameworkOptions {
            commands: vec![
                age(),
                register(),
                ping(),
                user_age(),
                author_age(),
                twitch(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(config) }));

    framework.run().await.unwrap();

    Ok(())
}
