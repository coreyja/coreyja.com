use std::net::SocketAddr;

use axum::{
    extract::{Query, State},
    http::Uri,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use axum_macros::debug_handler;
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::try_join;
use tracing::{info, instrument, warn};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Layer, Registry};
use tracing_tree::HierarchicalLayer;

use color_eyre::Result;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
// User data, which is stored and accessible in all command invocations
struct Data {
    twitch_config: TwitchConfig,
}

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
    let url = generate_user_twitch_link(&ctx.data().twitch_config)?;

    ctx.say(format!("Twitch Verify: {url}")).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::from_default_env();
    let subscriber = Registry::default().with(
        HierarchicalLayer::new(2)
            .with_ansi(true)
            .with_verbose_entry(true)
            .with_verbose_exit(true)
            .with_bracketed_fields(true)
            .with_filter(filter),
    );
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let twitch_config = TwitchConfig::from_env()?;

    let discord_future = run_discord_bot(&twitch_config);
    let axum_future = run_axum(&twitch_config);

    try_join!(discord_future, axum_future)?;

    Ok(())
}

async fn run_discord_bot(twitch_config: &TwitchConfig) -> Result<()> {
    let twitch_config = twitch_config.clone();
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
        .user_data_setup(move |_ctx, _ready, _framework| {
            Box::pin(async move { Ok(Data { twitch_config }) })
        });

    framework.run().await.unwrap();

    Ok(())
}

async fn run_axum(twitch_config: &TwitchConfig) -> color_eyre::Result<()> {
    // build our application with a route
    let app = Router::with_state(twitch_config.clone())
        // `GET /` goes to `root`
        .route("/twitch_oauth", get(twitch_oauth));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

#[derive(Debug, Deserialize)]
struct TwitchOauthRequest {
    code: String,
    scope: String,
    state: Option<String>,
}

#[derive(Serialize)]
struct TwitchCodeExchangeRequest {
    client_id: String,
    client_secret: String,
    code: String,
    grant_type: String,
    redirect_uri: String,
}
// basic handler that responds with a static string
async fn twitch_oauth(
    Query(oauth): Query<TwitchOauthRequest>,
    State(twitch_config): State<TwitchConfig>,
) -> impl IntoResponse {
    let client = reqwest::Client::new();

    let token_response = client
        .post("https://id.twitch.tv/oauth2/token")
        .form(&TwitchCodeExchangeRequest {
            client_id: twitch_config.client_id.clone(),
            client_secret: twitch_config.client_secret.clone(),
            code: oauth.code.clone(),
            grant_type: "authorization_code".to_string(),
            redirect_uri: twitch_config.redirect_uri.clone(),
        })
        .send()
        .await
        .unwrap();

    let json = token_response.json::<TwitchTokenResponse>().await.unwrap();
    let access_token = json.access_token;

    let validate_response = client
        .get("https://id.twitch.tv/oauth2/validate")
        .bearer_auth(access_token)
        .send()
        .await
        .unwrap();

    let json = validate_response
        .json::<TwitchValidateResponse>()
        .await
        .unwrap();

    format!("{json:#?}")
}
#[derive(Serialize, Deserialize, Debug)]
struct TwitchTokenResponse {
    access_token: String,
    expires_in: i64,
    refresh_token: String,
    scope: Option<Vec<String>>,
    token_type: String,
}

fn generate_user_twitch_link(config: &TwitchConfig) -> Result<Uri> {
    let client_id = &config.client_id;
    let redirect_uri = &config.redirect_uri;

    Ok(Uri::builder()
        .scheme("https")
        .authority("id.twitch.tv")
        .path_and_query(format!("/oauth2/authorize?client_id={client_id}&redirect_uri={redirect_uri}&response_type=code&scope="))
        .build()?)
}

#[derive(Debug, Clone)]
struct TwitchConfig {
    client_id: String,
    client_secret: String,

    redirect_uri: String,
    bot_access_token: String,
}

impl TwitchConfig {
    fn from_env() -> Result<Self> {
        Ok(Self {
            client_id: std::env::var("TWITCH_CLIENT_ID")?,
            client_secret: std::env::var("TWITCH_CLIENT_SECRET")?,
            redirect_uri: std::env::var("TWITCH_REDIRECT_URI")?,
            bot_access_token: std::env::var("TWITCH_BOT_ACCESS_TOKEN")?,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct TwitchValidateResponse {
    client_id: String,
    expires_in: i64,
    login: String,
    scopes: Vec<String>,
    user_id: String,
}
