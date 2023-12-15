use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    response::Redirect,
};

use crate::state::AppState;

pub(crate) async fn google_auth(State(app_state): State<AppState>) -> Redirect {
    let redirect_uri = app_state.app.app_url("/admin/auth/google/callback");
    let auth_url = format!(
      "https://accounts.google.com/o/oauth2/v2/auth?response_type=code&client_id={}&redirect_uri={}&scope=email",
      app_state.google.client_id, redirect_uri
  );

    Redirect::to(&auth_url)
}

pub(crate) async fn google_auth_callback(
    State(app_state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<String, String> {
    // Extract the authorization code from the callback query parameters
    let code = match params.get("code") {
        Some(code) => code,
        None => return Err("No code found in the query string".into()),
    };

    // Prepare the request to exchange the code for an access token
    let client = reqwest::Client::new();
    let token_url = "https://oauth2.googleapis.com/token";
    let response = client
        .post(token_url)
        .form::<[(&str, &str)]>(&[
            ("client_id", &app_state.google.client_id),
            ("client_secret", &app_state.google.client_secret),
            ("code", code),
            ("grant_type", "authorization_code"),
            (
                "redirect_uri",
                &app_state.app.app_url("/admin/auth/google/callback"),
            ),
        ])
        .send()
        .await;

    let response = match response {
        Ok(res) => res,
        Err(_) => return Err("Failed to send request".into()),
    };

    let token_data: serde_json::Value = match response.json().await {
        Ok(data) => data,
        Err(_) => return Err("Failed to parse token response".into()),
    };

    // Extract the access token from the token data
    let access_token = match token_data["access_token"].as_str() {
        Some(token) => token,
        None => return Err("Access token not found".into()),
    };

    // Use the access token to retrieve some user information
    let userinfo_url = "https://www.googleapis.com/oauth2/v2/userinfo";
    let user_info_response = client
        .get(userinfo_url)
        .bearer_auth(access_token)
        .send()
        .await;

    let user_info = match user_info_response {
        Ok(res) => res
            .text()
            .await
            .unwrap_or_else(|_| "Failed to get user info".into()),
        Err(_) => "Failed to send user info request".into(),
    };

    Ok(user_info)
}
