use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use cja::{color_eyre::eyre::Context, server::session::Session};
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use tracing::warn;
use uuid::Uuid;

use crate::{
    encrypt::encrypt,
    http_server::{auth::session::DBSession, ResponseResult},
    linear::graphql::get_me,
    AppState,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinearOAuthInit {
    pub return_to: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinearOAuthCode {
    code: String,
    state: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct LinearOAuthTokenRequest {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    grant_type: String,
    code: String,
    actor: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct LinearOAuthTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: Option<u64>,
    scope: String,
}

#[axum_macros::debug_handler(state = AppState)]
pub(crate) async fn linear_auth(
    State(app_state): State<AppState>,
    Query(query): Query<LinearOAuthInit>,
    Session(_session): Session<DBSession>,
) -> impl IntoResponse {
    let state_id = Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO GithubLoginStates (github_login_state_id, state, return_to, app)
        VALUES ($1, 'created', $2, 'linear')
        "#,
        state_id,
        query.return_to.clone()
    )
    .execute(&app_state.db)
    .await?;

    let auth_url = format!(
        "https://linear.app/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&actor=app",
        app_state.linear.client_id,
        urlencoding::encode(&app_state.linear.redirect_uri),
        urlencoding::encode("read,write,admin,app:assignable,app:mentionable,issue:create,comment:create"),
        state_id
    );

    ResponseResult::Ok(Redirect::temporary(&auth_url))
}

#[axum_macros::debug_handler(state = AppState)]
#[allow(clippy::too_many_lines)]
pub(crate) async fn linear_callback(
    State(app_state): State<AppState>,
    Query(query): Query<LinearOAuthCode>,
    Session(_session): Session<DBSession>,
) -> impl IntoResponse {
    let Some(state) = query.state else {
        warn!("No state provided in Linear OAuth Redirect");
        return ResponseResult::Ok(Redirect::temporary("/"));
    };

    let state_record = sqlx::query!(
        r#"
        SELECT * FROM GithubLoginStates
        WHERE github_login_state_id = $1 AND state = 'created' AND app = 'linear'
        "#,
        state
    )
    .fetch_one(&app_state.db)
    .await?;

    let client = reqwest::Client::new();

    let token_request = LinearOAuthTokenRequest {
        client_id: app_state.linear.client_id.clone(),
        client_secret: app_state.linear.client_secret.clone(),
        redirect_uri: app_state.linear.redirect_uri.clone(),
        grant_type: "authorization_code".to_string(),
        code: query.code.clone(),
        actor: "app".to_string(),
    };

    let token_response: LinearOAuthTokenResponse = client
        .post("https://api.linear.app/oauth/token")
        .json(&token_request)
        .send()
        .await?
        .json()
        .await
        .wrap_err("Failed to exchange authorization code for access token")?;

    let me_data = get_me(&token_response.access_token)
        .await
        .wrap_err("Failed to fetch user and application information")?;

    let workspace_id = me_data.viewer.organization.id;
    let actor_id = me_data.viewer.id;

    let existing_installation = sqlx::query!(
        r#"
        SELECT linear_installation_id FROM linear_installations
        WHERE external_workspace_id = $1
        "#,
        workspace_id
    )
    .fetch_optional(&app_state.db)
    .await?;

    let expires_in = token_response
        .expires_in
        .ok_or_else(|| eyre!("No expiration time provided"))?
        .try_into()?;

    let expires_at = chrono::Utc::now() + chrono::Duration::seconds(expires_in);

    if let Some(installation) = existing_installation {
        sqlx::query!(
            r#"
            UPDATE linear_installations
            SET
                external_actor_id = $1,
                encrypted_access_token = $2,
                token_expires_at = $3,
                scopes = $4,
                updated_at = NOW()
            WHERE linear_installation_id = $5
            "#,
            actor_id,
            encrypt(&token_response.access_token, &app_state.encrypt_config)?,
            expires_at,
            &token_response
                .scope
                .split(',')
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>(),
            installation.linear_installation_id
        )
        .execute(&app_state.db)
        .await?;
    } else {
        sqlx::query!(
            r#"
            INSERT INTO linear_installations (
                linear_installation_id,
                external_workspace_id,
                external_actor_id,
                encrypted_access_token,
                token_expires_at,
                scopes
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            Uuid::new_v4(),
            workspace_id,
            actor_id,
            encrypt(&token_response.access_token, &app_state.encrypt_config)?,
            expires_at,
            &token_response
                .scope
                .split(',')
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
        )
        .execute(&app_state.db)
        .await?;
    }

    sqlx::query!(
        r#"
        UPDATE GithubLoginStates
        SET state = 'linear_completed'
        WHERE github_login_state_id = $1 AND state = 'created'
        "#,
        state
    )
    .execute(&app_state.db)
    .await?;

    let return_to = state_record
        .return_to
        .unwrap_or_else(|| "/admin".to_string());
    ResponseResult::Ok(Redirect::temporary(&return_to))
}
