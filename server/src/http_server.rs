use crate::*;

use axum::{
    extract::{FromRef, Query, State},
    response::IntoResponse,
    routing::{get},
    Router, Server,
};

impl FromRef<Config> for TwitchConfig {
    fn from_ref(config: &Config) -> Self {
        config.twitch.clone()
    }
}

pub(crate) async fn run_axum(config: Config) -> color_eyre::Result<()> {
    // build our application with a route
    let app = Router::with_state(config)
        // `GET /` goes to `root`
        .route("/twitch_oauth", get(twitch_oauth));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
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
