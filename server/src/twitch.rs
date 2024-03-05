use crate::{Deserialize, IntoDiagnostic, Result, Serialize};
use tracing::instrument;

#[derive(Debug, Deserialize)]
pub(crate) struct TwitchOauthRequest {
    pub code: String,
    pub scope: String,
    pub state: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct TwitchCodeExchangeRequest {
    pub client_id: String,
    pub client_secret: String,
    pub code: String,
    pub grant_type: String,
    pub redirect_uri: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TwitchTokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub refresh_token: String,
    pub scope: Option<Vec<String>>,
    pub token_type: String,
}

#[derive(Debug, Clone)]
pub(crate) struct TwitchConfig {
    pub client_id: String,
    pub client_secret: String,

    pub bot_access_token: Option<String>,

    pub channel_user_id: String,
    pub bot_user_id: String,
}

impl TwitchConfig {
    #[instrument(name = "TwitchConfig::from_env")]
    pub(crate) fn from_env() -> Result<Self> {
        Ok(Self {
            client_id: std::env::var("TWITCH_CLIENT_ID").into_diagnostic()?,
            client_secret: std::env::var("TWITCH_CLIENT_SECRET").into_diagnostic()?,
            bot_access_token: std::env::var("TWITCH_BOT_ACCESS_TOKEN").ok(),
            bot_user_id: std::env::var("TWITCH_BOT_USER_ID").into_diagnostic()?,
            channel_user_id: std::env::var("TWITCH_CHANNEL_USER_ID").into_diagnostic()?,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TwitchValidateResponse {
    pub client_id: String,
    pub expires_in: i64,
    pub login: String,
    pub scopes: Vec<String>,
    pub user_id: String,
}

pub(crate) async fn get_chatters(config: &TwitchConfig) -> Result<TwitchChattersPage> {
    let client = reqwest::Client::new();

    let broadcaster_id = &config.channel_user_id;
    let mod_id = &config.bot_user_id;
    let response = client
        .get(format!("https://api.twitch.tv/helix/chat/chatters?broadcaster_id={broadcaster_id}&moderator_id={mod_id}"))
        .bearer_auth(config.bot_access_token.as_ref().expect("We need a bot access token here. This was required and then it was hard to generate for prod and I was lazy and we aren't using this yet so :shrug:"))
        .header("Client-Id", &config.client_id)
        .send()
        .await.into_diagnostic()
        ?;

    response
        .json::<TwitchChattersPage>()
        .await
        .into_diagnostic()
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Chatters {
    pub(crate) user_login: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Pagination {
    cursor: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TwitchChattersPage {
    pub(crate) data: Vec<Chatters>,
    pub(crate) pagination: Pagination,
    pub(crate) total: i64,
}
