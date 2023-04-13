use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use chrono::{Duration, Utc};

use crate::{http_server::errors::MietteError, *};

pub(crate) async fn handler(
    Query(oauth): Query<TwitchOauthRequest>,
    State(config): State<Config>,
) -> Result<impl IntoResponse, MietteError> {
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
        .await
        .into_diagnostic()?;

    let token_json = token_response
        .json::<TwitchTokenResponse>()
        .await
        .into_diagnostic()?;
    let access_token = &token_json.access_token;

    let validate_response = client
        .get("https://id.twitch.tv/oauth2/validate")
        .bearer_auth(access_token)
        .send()
        .await
        .into_diagnostic()?;

    let json = validate_response
        .json::<TwitchValidateResponse>()
        .await
        .into_diagnostic()?;

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
        .await
        .into_diagnostic()?
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
        .await
        .into_diagnostic()?;

        sqlx::query!(
            "UPDATE UserTwitchLinkStates
            SET
                status = 'used' AND
                updated_at = CURRENT_TIMESTAMP
            WHERE state = $1",
            state
        )
        .execute(&config.db_pool)
        .await
        .into_diagnostic()?;
    }

    Ok(format!("{json:#?}"))
}
