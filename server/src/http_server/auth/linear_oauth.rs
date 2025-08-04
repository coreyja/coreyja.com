use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use cja::{color_eyre::eyre::Context, server::session::Session};
use serde::{Deserialize, Serialize};
use tracing::warn;
use uuid::Uuid;

use crate::{
    encrypt::encrypt,
    http_server::{auth::session::DBSession, ResponseResult},
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

#[derive(Debug, Clone, Deserialize, Serialize)]
struct LinearWorkspaceResponse {
    data: LinearWorkspaceData,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct LinearWorkspaceData {
    viewer: LinearViewer,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct LinearViewer {
    organization: LinearOrganization,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct LinearOrganization {
    id: String,
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

    let workspace_response: LinearWorkspaceResponse = client
        .post("https://api.linear.app/graphql")
        .header("Authorization", token_response.access_token.to_string())
        .json(&serde_json::json!({
            "query": r"
                query {
                    viewer {
                        organization {
                            id
                        }
                    }
                }
            "
        }))
        .send()
        .await?
        .json()
        .await
        .wrap_err("Failed to fetch workspace information")?;

    let workspace_id = workspace_response.data.viewer.organization.id;

    let existing_installation = sqlx::query!(
        r#"
        SELECT linear_installation_id FROM linear_installations
        WHERE external_workspace_id = $1
        "#,
        workspace_id
    )
    .fetch_optional(&app_state.db)
    .await?;

    if let Some(installation) = existing_installation {
        sqlx::query!(
            r#"
            UPDATE linear_installations
            SET
                encrypted_access_token = $1,
                token_expires_at = $2,
                scopes = $3,
                updated_at = NOW()
            WHERE linear_installation_id = $4
            "#,
            encrypt(&token_response.access_token, &app_state.encrypt_config)?,
            token_response
                .expires_in
                .map(|exp| chrono::Utc::now() + chrono::Duration::seconds(exp as i64)),
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
                encrypted_access_token,
                token_expires_at,
                scopes
            )
            VALUES ($1, $2, $3, $4, $5)
            "#,
            Uuid::new_v4(),
            workspace_id,
            encrypt(&token_response.access_token, &app_state.encrypt_config)?,
            token_response
                .expires_in
                .map(|exp| chrono::Utc::now() + chrono::Duration::seconds(exp as i64)),
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
