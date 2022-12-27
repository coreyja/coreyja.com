use crate::*;

use axum::{
    extract::{FromRef, Query, State},
    response::IntoResponse,
    routing::get,
    Router, Server,
};
use sqlx::query;

impl FromRef<Config> for TwitchConfig {
    fn from_ref(config: &Config) -> Self {
        config.twitch.clone()
    }
}

pub(crate) async fn run_axum(config: Config) -> color_eyre::Result<()> {
    let app = Router::with_state(config)
        .route("/twitch_oauth", get(twitch_oauth))
        .route("/github_oauth", get(github_oauth));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn twitch_oauth(
    Query(oauth): Query<TwitchOauthRequest>,
    State(config): State<Config>,
) -> impl IntoResponse {
    let twitch_config = config.twitch;
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

    if let Some(state) = oauth.state {
        let discord_user_id = sqlx::query!(
            "SELECT discord_user_id FROM TwitchLinkStates WHERE state = $1",
            state
        )
        .fetch_one(&config.db_pool)
        .await
        .unwrap()
        .discord_user_id;

        sqlx::query!(
            "INSERT INTO DiscordTwitchLinks (discord_user_id, twitch_user_id, twitch_login) VALUES ($1, $2, $3)",
            discord_user_id,
            json.user_id,
            json.login
        )
        .execute(&config.db_pool)
        .await
        .unwrap();
    }

    format!("{json:#?}")
}

#[derive(Debug, Deserialize)]
pub(crate) struct GithubOauthRequest {
    pub code: String,
    pub state: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct GithubCodeExchangeRequest {
    pub client_id: String,
    pub client_secret: String,
    pub code: String,
    pub redirect_uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GithubTokenResponse {
    pub(crate) access_token: String,
    pub(crate) expires_in: i64,
    pub(crate) refresh_token: String,
    pub(crate) refresh_token_expires_in: i64,
    pub(crate) scope: String,
    pub(crate) token_type: String,
}

async fn github_oauth(
    Query(oauth): Query<GithubOauthRequest>,
    State(config): State<Config>,
) -> impl IntoResponse {
    let client = reqwest::Client::new();
    let github = config.github;

    let token_response = client
        .post("https://github.com/login/oauth/access_token")
        .json(&GithubCodeExchangeRequest {
            client_id: github.client_id.clone(),
            client_secret: github.client_secret.clone(),
            code: oauth.code.clone(),
            redirect_uri: github.redirect_uri.clone(),
        })
        .send()
        .await
        .unwrap();
    let text = token_response.text().await.unwrap();
    let token_response: GithubTokenResponse = serde_urlencoded::from_str(&text).unwrap();

    let state = oauth
        .state
        .expect("Github oauth should always come back with a state when we kick it off");

    let discord_user_id = sqlx::query!(
        "SELECT discord_user_id FROM GithubLinkStates WHERE state = $1",
        state
    )
    .fetch_one(&config.db_pool)
    .await
    .expect(indoc::indoc! {"
        If there was a state from Githun oauth, it should exist in our DB.
        Did this oauth get triggered by someone else with a state we don't know about?
    "})
    .discord_user_id;

    format!("{token_response:#?}\n\n{discord_user_id}")
}
