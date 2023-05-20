use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::{http_server::errors::MietteError, *};

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

pub(crate) async fn handler(
    Query(oauth): Query<GithubOauthRequest>,
    State(config): State<AppState>,
) -> Result<impl IntoResponse, MietteError> {
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
        .await
        .into_diagnostic()?;
    let text = token_response.text().await.into_diagnostic()?;
    let token_response: GithubTokenResponse =
        serde_urlencoded::from_str(&text).into_diagnostic()?;

    let state = oauth.state.ok_or_else(|| {
        miette::miette!("Github oauth should always come back with a state when we kick it off")
    })?;

    let user_id = sqlx::query!(
        "SELECT user_id FROM UserGithubLinkStates WHERE state = $1 AND status = 'pending'",
        state
    )
    .fetch_one(&config.db_pool)
    .await
    .into_diagnostic()
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
    .await
    .into_diagnostic()?;

    sqlx::query!(
        "UPDATE UserGithubLinkStates
            SET
                status = 'used' AND
                updated_at = CURRENT_TIMESTAMP
            WHERE state = $1",
        state
    )
    .execute(&config.db_pool)
    .await
    .into_diagnostic()?;

    Ok(format!("{token_response:#?}"))
}
