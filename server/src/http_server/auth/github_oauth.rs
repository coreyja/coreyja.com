use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use db::users::UserFromDB;
use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::{encrypt::encrypt, http_server::ResponseResult, AppState};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitHubOAuthCode {
    code: String,
    state: Option<Uuid>,
}

typify::import_types!("src/http_server/auth/github_token_response.schema.json");

impl GithubUser {
    fn login(&self) -> &str {
        match &self {
            GithubUser::PrivateUser { login, .. } => login,
            GithubUser::PublicUser { login, .. } => login,
        }
    }

    fn id(&self) -> &str {
        match &self {
            GithubUser::PrivateUser { node_id, .. } => node_id,
            GithubUser::PublicUser { node_id, .. } => node_id,
        }
    }
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
    State(app_state): State<AppState>,
    Query(query): Query<GitHubOAuthCode>,
    cookies: Cookies,
) -> impl IntoResponse {
    let client = reqwest::Client::new();

    let oauth_response: Value = client
        .post("https://github.com/login/oauth/access_token")
        .query(&[
            ("client_id", &app_state.github.client_id),
            ("client_secret", &app_state.github.client_secret),
            ("code", &query.code),
            ("redirect_uri", &app_state.app.app_url("/auth/github_oauth")),
        ])
        .header("Accept", "application/json")
        .send()
        .await
        .into_diagnostic()?
        .json()
        .await
        .into_diagnostic()?;

    let oauth_response: GitHubOAuthResponse =
        serde_json::from_value(oauth_response).into_diagnostic()?;

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
        .into_diagnostic()?
        .json::<Value>()
        .await
        .into_diagnostic()?;

    let user_info: GithubUser = serde_json::from_value(user_info).into_diagnostic()?;
    let pool = &app_state.db;

    let (local_user, github_link_id): (UserFromDB, Uuid) = {
        let github_user = &user_info;
        let user = db::sqlx::query!(
            r#"
        SELECT Users.*, GithubLinks.github_link_id
        FROM Users
        JOIN GithubLinks USING (user_id)
        WHERE GithubLinks.external_github_login = $1
        "#,
            github_user.login()
        )
        .fetch_optional(pool)
        .await
        .into_diagnostic()?;

        if let Some(user) = user {
            sqlx::query!(
                r#"
            UPDATE GithubLinks
            SET
                encrypted_access_token = $1,
                encrypted_refresh_token = $2,
                access_token_expires_at = $3,
                refresh_token_expires_at = $4,
                external_github_login = $5
            WHERE github_link_id = $6
            "#,
                encrypt(&oauth_response.access_token, &app_state.encrypt_config)?,
                encrypt(&oauth_response.refresh_token, &app_state.encrypt_config)?,
                chrono::Utc::now()
                    + chrono::Duration::seconds(
                        oauth_response.expires_in.try_into().into_diagnostic()?
                    ),
                chrono::Utc::now()
                    + chrono::Duration::seconds(
                        oauth_response
                            .refresh_token_expires_in
                            .try_into()
                            .into_diagnostic()?
                    ),
                github_user.login(),
                user.github_link_id
            )
            .execute(pool)
            .await
            .into_diagnostic()?;

            (
                db::users::UserFromDB {
                    user_id: user.user_id,
                    created_at: user.created_at,
                    updated_at: user.updated_at,
                },
                user.github_link_id,
            )
        } else {
            let user = db::sqlx::query_as!(
                db::users::UserFromDB,
                r#"
            INSERT INTO Users (user_id)
            VALUES ($1)
            RETURNING *
            "#,
                uuid::Uuid::new_v4()
            )
            .fetch_one(pool)
            .await
            .into_diagnostic()?;

            let link = db::sqlx::query!(
                r#"
            INSERT INTO GithubLinks (
                github_link_id,
                user_id,
                external_github_id,
                external_github_login,
                encrypted_access_token,
                encrypted_refresh_token,
                access_token_expires_at,
                refresh_token_expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING github_link_id
            "#,
                uuid::Uuid::new_v4(),
                user.user_id,
                github_user.id().to_string(),
                github_user.login(),
                encrypt(&oauth_response.access_token, &app_state.encrypt_config)?,
                encrypt(&oauth_response.refresh_token, &app_state.encrypt_config)?,
                chrono::Utc::now()
                    + chrono::Duration::seconds(
                        oauth_response.expires_in.try_into().into_diagnostic()?
                    ),
                chrono::Utc::now()
                    + chrono::Duration::seconds(
                        oauth_response
                            .refresh_token_expires_in
                            .try_into()
                            .into_diagnostic()?
                    ),
            )
            .fetch_one(pool)
            .await
            .into_diagnostic()?;

            (user, link.github_link_id)
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
    .fetch_one(&app_state.db)
    .await
    .into_diagnostic()?;

    let private = cookies.private(&app_state.cookie_key.0);

    let session_cookie =
        tower_cookies::Cookie::build(("session_id", session.session_id.to_string()))
            .path("/")
            .http_only(true)
            .secure(true)
            .expires(None);
    private.add(session_cookie.into());

    if let Some(state) = query.state {
        let state = sqlx::query!(
            r#"
        UPDATE GithubLoginStates
        SET state = $1, github_link_id = $2
        WHERE github_login_state_id = $3
        RETURNING *
        "#,
            "github_completed",
            github_link_id,
            &state
        )
        .fetch_one(pool)
        .await
        .into_diagnostic()?;

        let projects = app_state.projects.clone();
        let project = projects
            .projects
            .iter()
            .find(|p| p.slug().unwrap() == state.app);

        if let Some(project) = project {
            if let Some(login_callback) = &project.frontmatter.login_callback {
                return ResponseResult::Ok(Redirect::temporary(&format!(
                    "{}?state={}",
                    login_callback, state.github_login_state_id
                )));
            }
        }
    }

    ResponseResult::Ok(Redirect::temporary("/"))
}