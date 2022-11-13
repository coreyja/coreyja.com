use axum::http::Uri;

use crate::*;

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

pub(crate) fn generate_user_twitch_link(config: &TwitchConfig) -> Result<Uri> {
    let client_id = &config.client_id;
    let redirect_uri = &config.redirect_uri;

    Ok(Uri::builder()
        .scheme("https")
        .authority("id.twitch.tv")
        .path_and_query(format!("/oauth2/authorize?client_id={client_id}&redirect_uri={redirect_uri}&response_type=code&scope="))
        .build()?)
}

#[derive(Debug, Clone)]
pub(crate) struct TwitchConfig {
    pub client_id: String,
    pub client_secret: String,

    pub redirect_uri: String,
    pub bot_access_token: String,

    pub channel_user_id: String,
    pub bot_user_id: String,
}

impl TwitchConfig {
    pub(crate) fn from_env() -> Result<Self> {
        Ok(Self {
            client_id: std::env::var("TWITCH_CLIENT_ID")?,
            client_secret: std::env::var("TWITCH_CLIENT_SECRET")?,
            redirect_uri: std::env::var("TWITCH_REDIRECT_URI")?,
            bot_access_token: std::env::var("TWITCH_BOT_ACCESS_TOKEN")?,
            bot_user_id: std::env::var("TWITCH_BOT_USER_ID")?,
            channel_user_id: std::env::var("TWITCH_CHANNEL_USER_ID")?,
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

pub(crate) async fn get_chatters(config: &TwitchConfig) -> impl std::fmt::Debug {
    let client = reqwest::Client::new();

    let broadcaster_id = &config.channel_user_id;
    let mod_id = &config.bot_user_id;
    let response = client
        .get(format!("https://api.twitch.tv/helix/chat/chatters?broadcaster_id={broadcaster_id}&moderator_id={mod_id}"))
        .bearer_auth(&config.bot_access_token)
        .header("Client-Id", &config.client_id)
        .send()
        .await
        .unwrap();

    let json = response.json::<TwitchChattersPage>().await.unwrap();

    json
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Chatters {
    user_login: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Pagination {
    cursor: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TwitchChattersPage {
    data: Vec<Chatters>,
    pagination: Pagination,
    total: i64,
}
