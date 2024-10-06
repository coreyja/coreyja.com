use std::sync::Arc;

use ::serenity::all::CacheHttp;
use color_eyre::eyre::Context as _;
use poise::serenity_prelude as serenity;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(prefix_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[derive(Clone, Debug)]
pub(crate) struct DiscordClient {
    pub http: Arc<serenity::Http>,
    cache: Arc<serenity::Cache>,
}

impl CacheHttp for DiscordClient {
    fn http(&self) -> &serenity::Http {
        self.http.as_ref()
    }

    fn cache(&self) -> Option<&Arc<serenity::Cache>> {
        Some(&self.cache)
    }
}

pub(crate) struct DiscordBot(serenity::Client);

impl DiscordBot {
    pub async fn run(mut self) -> cja::Result<()> {
        self.0.start().await.wrap_err("Error running discord bot")?;

        Ok(())
    }
}

pub(crate) struct DiscordSetup {
    pub bot: DiscordBot,
    pub client: DiscordClient,
}

pub(crate) async fn setup() -> cja::Result<DiscordSetup> {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age(), register()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .wrap_err("Could not build serenity Discord client")?;

    let outside_client = DiscordClient {
        http: client.http.clone(),
        cache: client.cache.clone(),
    };

    Ok(DiscordSetup {
        bot: DiscordBot(client),
        client: outside_client,
    })
}
