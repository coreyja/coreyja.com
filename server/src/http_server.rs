use crate::*;

use axum::{
    extract::{FromRef, Query, State},
    response::IntoResponse,
    routing::{get, post},
    Router, Server,
};
use chrono::Duration;
use color_eyre::eyre::{Context, ContextCompat};
use maud::{html, Markup};
use sqlx::types::chrono::Utc;

impl FromRef<Config> for TwitchConfig {
    fn from_ref(config: &Config) -> Self {
        config.twitch.clone()
    }
}

impl FromRef<Config> for AppConfig {
    fn from_ref(config: &Config) -> Self {
        config.app.clone()
    }
}

async fn home_page() -> Markup {
    html! {
        p {
            "Hello! You stumbled upon the beta version for my personal site. To see the live version, go to "
            a href="https://coreyja.com" { "coreyja.com" }
        }

        p {
            "Right now this is mostly powering a personal Discord bot. In the future it will be the home for everying `coreyja` branded!"
        }
    }
}

pub(crate) async fn run_axum(config: Config) -> color_eyre::Result<()> {
    let app = Router::new()
        .route("/", get(home_page))
        .route("/twitch_oauth", get(twitch_oauth))
        .route("/github_oauth", get(github_oauth))
        .route(
            "/admin/upwork/proposals/:id",
            get(admin::upwork_proposal_get),
        )
        .route(
            "/admin/upwork/proposals/:id",
            post(admin::upwork_proposal_post),
        )
        .with_state(config);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}

async fn twitch_oauth(
    Query(oauth): Query<TwitchOauthRequest>,
    State(config): State<Config>,
) -> Result<impl IntoResponse, EyreError> {
    let twitch_config = config.twitch;
    let client = reqwest::Client::new();

    let redirect_uri = format!("{base_url}/twitch_oauth", base_url = config.app.base_url);

    let token_response = client
        .post("https://id.twitch.tv/oauth2/token")
        .form(&TwitchCodeExchangeRequest {
            client_id: twitch_config.client_id.clone(),
            client_secret: twitch_config.client_secret.clone(),
            code: oauth.code.clone(),
            grant_type: "authorization_code".to_string(),
            redirect_uri,
        })
        .send()
        .await?;

    let token_json = token_response.json::<TwitchTokenResponse>().await?;
    let access_token = &token_json.access_token;

    let validate_response = client
        .get("https://id.twitch.tv/oauth2/validate")
        .bearer_auth(access_token)
        .send()
        .await?;

    let json = validate_response.json::<TwitchValidateResponse>().await?;

    if let Some(state) = oauth.state {
        let user_id = sqlx::query!(
            "
            SELECT user_id
            FROM UserTwitchLinkStates
            WHERE  
                state = $1 AND
                status = 'pending'
            ",
            state
        )
        .fetch_one(&config.db_pool)
        .await?
        .user_id;
        let now = Utc::now();
        let expires_at = Utc::now() + Duration::seconds(token_json.expires_in);

        sqlx::query!(
            "INSERT INTO
            UserTwitchLinks
                (
                    user_id,
                    external_twitch_user_id,
                    external_twitch_login,
                    access_token,
                    refresh_token,
                    access_token_expires_at,
                    access_token_validated_at
                )
            VALUES ($1, $2, $3, $4, $5, $6, $7)",
            user_id,
            json.user_id,
            json.login,
            token_json.access_token,
            token_json.refresh_token,
            expires_at,
            now
        )
        .execute(&config.db_pool)
        .await?;

        sqlx::query!(
            "UPDATE UserTwitchLinkStates
            SET
                status = 'used' AND
                updated_at = CURRENT_TIMESTAMP
            WHERE state = $1",
            state
        )
        .execute(&config.db_pool)
        .await?;
    }

    Ok(format!("{json:#?}"))
}

#[derive(Debug, Deserialize)]
pub(crate) struct GithubOauthRequest {
    pub(crate) code: String,
    pub(crate) state: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct GithubCodeExchangeRequest {
    pub(crate) client_id: String,
    pub(crate) client_secret: String,
    pub(crate) code: String,
    pub(crate) redirect_uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GithubTokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub refresh_token: String,
    pub refresh_token_expires_in: i64,
    pub scope: String,
    pub token_type: String,
}

async fn github_oauth(
    Query(oauth): Query<GithubOauthRequest>,
    State(config): State<Config>,
) -> Result<impl IntoResponse, EyreError> {
    let client = reqwest::Client::new();
    let github = &config.github;
    let redirect_uri = github_redirect_uri(&config);

    let token_response = client
        .post("https://github.com/login/oauth/access_token")
        .json(&GithubCodeExchangeRequest {
            client_id: github.client_id.clone(),
            client_secret: github.client_secret.clone(),
            code: oauth.code.clone(),
            redirect_uri,
        })
        .send()
        .await?;
    let text = token_response.text().await?;
    let token_response: GithubTokenResponse = serde_urlencoded::from_str(&text)?;

    let state = oauth
        .state
        .wrap_err("Github oauth should always come back with a state when we kick it off")?;

    let user_id = sqlx::query!(
        "SELECT user_id FROM UserGithubLinkStates WHERE state = $1 AND status = 'pending'",
        state
    )
    .fetch_one(&config.db_pool)
    .await
    .wrap_err(indoc::indoc! {"
        If there was a state from Githun oauth, it should exist in our DB.
        Did this oauth get triggered by someone else with a state we don't know about?
        Or is the status not pending anymore?
    "})?
    .user_id;

    let username: String = "test".into();

    let now = Utc::now();
    let expires_at = now + Duration::seconds(token_response.expires_in);
    let refresh_expires_at = now + Duration::seconds(token_response.refresh_token_expires_in);

    sqlx::query!(
        "
        INSERT INTO
            UserGithubLinks (
                user_id,
                external_github_username,
                access_token,
                refresh_token,
                access_token_expires_at,
                refresh_token_expires_at
            )
        VALUES ($1, $2, $3, $4, $5, $6)
        ",
        user_id,
        username,
        token_response.access_token,
        token_response.refresh_token,
        expires_at,
        refresh_expires_at,
    )
    .execute(&config.db_pool)
    .await?;

    sqlx::query!(
        "UPDATE UserGithubLinkStates
            SET
                status = 'used' AND
                updated_at = CURRENT_TIMESTAMP
            WHERE state = $1",
        state
    )
    .execute(&config.db_pool)
    .await?;

    Ok(format!("{token_response:#?}"))
}

pub struct EyreError(color_eyre::Report);

impl IntoResponse for EyreError {
    fn into_response(self) -> axum::response::Response {
        self.0.to_string().into_response()
    }
}

impl<T> From<T> for EyreError
where
    T: Into<color_eyre::Report>,
{
    fn from(err: T) -> Self {
        EyreError(err.into())
    }
}

mod admin;
