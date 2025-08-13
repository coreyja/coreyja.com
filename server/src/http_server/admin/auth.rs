use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use serde::{Deserialize, Serialize};

use crate::{http_server::auth::session::AdminUser, state::AppState};

pub(crate) async fn google_auth(State(app_state): State<AppState>, _: AdminUser) -> Redirect {
    let redirect_uri = app_state.app.app_url("/admin/auth/google/callback");
    let scope = &["https://www.googleapis.com/auth/youtube", "email"].join(" ");
    let auth_url = url::Url::parse_with_params(
        "https://accounts.google.com/o/oauth2/v2/auth",
        &[
            ("response_type", "code"),
            ("client_id", &app_state.google.client_id),
            ("redirect_uri", &redirect_uri),
            ("scope", scope),
            ("access_type", "offline"),
            ("include_granted_scopes", "true"),
        ],
    )
    .unwrap()
    .to_string();

    Redirect::to(&auth_url)
}

struct GoogleAuthCallbackQuery {
    code: String,
    state: String,
}

pub(crate) async fn google_auth_callback(
    State(app_state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    admin: AdminUser,
) -> Result<impl IntoResponse, String> {
    // Extract the authorization code from the callback query parameters
    let Some(code) = params.get("code") else {
        return Err("No code found in the query string".into());
    };

    // Prepare the request to exchange the code for an access token
    let client = reqwest::Client::new();
    let token_url = "https://oauth2.googleapis.com/token";
    let response = client
        .post(token_url)
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", &app_state.google.client_id),
            ("client_secret", &app_state.google.client_secret),
            ("code", code),
            (
                "redirect_uri",
                &app_state.app.app_url("/admin/auth/google/callback"),
            ),
        ])
        .send()
        .await;

    let Ok(response) = response else {
        return Err("Failed to send request".into());
    };

    // https://developers.google.com/identity/protocols/oauth2/web-server#httprest_3
    #[derive(serde::Deserialize, Debug)]
    struct TokenData {
        access_token: String,
        expires_in: i64,
        refresh_token: String,
        scope: String,
        token_type: String,
    }

    let token_data: TokenData = match response.json().await {
        Ok(data) => data,
        Err(_) => return Err("Failed to parse token response".into()),
    };

    // Use the access token to retrieve some user information
    let userinfo_url = "https://www.googleapis.com/oauth2/v2/userinfo";
    let user_info_response = client
        .get(userinfo_url)
        .bearer_auth(&token_data.access_token)
        .send()
        .await;

    #[derive(Serialize, Deserialize)]
    struct UserInfo {
        email: String,
        id: String,
        name: String,
        picture: String,
        verified_email: bool,
    }

    let user_info = user_info_response
        .unwrap()
        .json::<UserInfo>()
        .await
        .unwrap();

    sqlx::query!(
        r#"
        INSERT INTO GoogleUsers (google_user_id, user_id, external_google_id, external_google_email, encrypted_access_token, access_token_expires_at, encrypted_refresh_token, scope)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        uuid::Uuid::new_v4(),
        admin.session.0.user_id,
        user_info.id,
        user_info.email,
        app_state.encrypt_config.encrypt(&token_data.access_token).unwrap(),
        chrono::Utc::now() + chrono::Duration::seconds(token_data.expires_in),
        app_state.encrypt_config.encrypt(&token_data.refresh_token).unwrap(),
        token_data.scope,
    )
    .execute(&app_state.db)
    .await
    .unwrap();

    Ok(Redirect::to("/admin"))
}
