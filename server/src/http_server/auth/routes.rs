use std::{borrow::Cow, sync::Arc};

use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, Query, State},
    http,
    response::{IntoResponse, Redirect},
};
use db::{sqlx, PgPool};
use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower_cookies::Cookies;
use typify::import_types;

use crate::AppState;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitHubOAuthCode {
    code: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct GitHubOAuthResponse {
    access_token: String,
    expires_in: u64,
    refresh_token: String,
    refresh_token_expires_in: u64,
    scope: String,
    token_type: String,
}

pub(crate) async fn github_oauth(
    State(state): State<AppState>,
    Query(code): Query<GitHubOAuthCode>,
    cookies: Cookies,
) -> impl IntoResponse {
    let client = reqwest::Client::new();

    let oauth_response: Value = client
        .post("https://github.com/login/oauth/access_token")
        .query(&[
            ("client_id", &state.github.client_id),
            ("client_secret", &state.github.client_secret),
            ("code", &code.code),
            ("redirect_uri", &state.app.app_url("/auth/github_oauth")),
        ])
        .header("Accept", "application/json")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let oauth_response: GitHubOAuthResponse = serde_json::from_value(oauth_response).unwrap();

    let user_info = client
        .get("https://api.github.com/user")
        .header("User-Agent", "github.com/coreyja/coreyja.com")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("Accept", "application/vnd.github+json")
        .header(
            "Authorization",
            format!("Bearer {}", oauth_response.access_token),
        )
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    let user_info: GithubUser = serde_json::from_value(user_info).unwrap();

    let local_user = {
        let pool = &state.db;
        let github_user = &user_info;
        let user = db::sqlx::query_as!(
            db::users::User,
            r#"
        SELECT Users.*
        FROM Users
        JOIN GithubLinks USING (user_id)
        WHERE GithubLinks.external_github_username = $1
        "#,
            github_user.login()
        )
        .fetch_optional(pool)
        .await
        .into_diagnostic()
        .unwrap();

        if let Some(user) = user {
            user
        } else {
            let user = db::sqlx::query_as!(
                db::users::User,
                r#"
            INSERT INTO Users (user_id)
            VALUES ($1)
            RETURNING *
            "#,
                uuid::Uuid::new_v4()
            )
            .fetch_one(pool)
            .await
            .into_diagnostic()
            .unwrap();

            db::sqlx::query!(
                r#"
            INSERT INTO GithubLinks (
                github_link_id,
                user_id,
                external_github_username,
                access_token,
                refresh_token,
                access_token_expires_at,
                refresh_token_expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
                uuid::Uuid::new_v4(),
                user.user_id,
                github_user.login(),
                oauth_response.access_token,
                oauth_response.refresh_token,
                chrono::Utc::now()
                    + chrono::Duration::seconds(oauth_response.expires_in.try_into().unwrap()),
                chrono::Utc::now()
                    + chrono::Duration::seconds(
                        oauth_response.refresh_token_expires_in.try_into().unwrap()
                    ),
            )
            .execute(pool)
            .await
            .into_diagnostic()
            .unwrap();

            user
        }
    };

    let session = sqlx::query!(
        r#"
    INSERT INTO Sessions (session_id, user_id)
    VALUES ($1, $2)
    RETURNING *
    "#,
        uuid::Uuid::new_v4(),
        local_user.user_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap();
    // let cookies = Cookies::from_request_parts(parts, state).await.unwrap();

    let private = cookies.private(&state.cookie_key.0);

    let session_cookie = tower_cookies::Cookie::build("session_id", session.session_id.to_string())
        .path("/")
        .http_only(true)
        .secure(true)
        .finish();
    private.add(session_cookie);

    format!("Logged in as {}", local_user.user_id)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct DBSession {
    session_id: uuid::Uuid,
    user_id: uuid::Uuid,
    updated_at: chrono::DateTime<chrono::Utc>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
impl FromRequestParts<AppState> for DBSession {
    type Rejection = axum::response::Redirect;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookies = Cookies::from_request_parts(parts, state).await.unwrap();

        let private = cookies.private(&state.cookie_key.0);

        let session_cookie = private.get("session_id");

        let Some(session_cookie) = session_cookie else {
            Err(Redirect::temporary("/_caje/auth"))?
        };
        let session_id = session_cookie.value().to_string();

        let session = sqlx::query_as!(
            DBSession,
            r#"
        SELECT *
        FROM Sessions
        WHERE session_id = $1
        "#,
            uuid::Uuid::parse_str(&session_id).unwrap()
        )
        .fetch_one(&state.db)
        .await
        .unwrap();

        Ok(session)
    }
}

import_types!("src/http_server/auth/github_token_response.schema.json");

impl GithubUser {
    fn login(&self) -> &str {
        match &self {
            GithubUser::PrivateUser { login, .. } => login,
            GithubUser::PublicUser { login, .. } => login,
        }
    }
}