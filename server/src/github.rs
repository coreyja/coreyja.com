use crate::*;

use axum::http::Uri;
use miette::Result;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub(crate) struct GithubConfig {
    pub(crate) app_id: u64,
    pub(crate) client_id: String,
    pub(crate) client_secret: String,
}

impl GithubConfig {
    pub(crate) fn from_env() -> Result<Self> {
        Ok(Self {
            app_id: std::env::var("GITHUB_APP_ID")
                .into_diagnostic()?
                .parse()
                .into_diagnostic()?,
            client_id: std::env::var("GITHUB_APP_CLIENT_ID").into_diagnostic()?,
            client_secret: std::env::var("GITHUB_APP_CLIENT_SECRET").into_diagnostic()?,
        })
    }
}

pub(crate) async fn generate_user_github_link(config: &AppState, user_id: i64) -> Result<Uri> {
    let client_id = &config.github.client_id;
    let redirect_uri = github_redirect_uri(config);

    let state = Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO UserGithubLinkStates (user_id, state) VALUES (?, ?)",
        user_id,
        state,
    )
    .execute(&config.db_pool)
    .await
    .into_diagnostic()?;

    Uri::builder()
        .scheme("https")
        .authority("github.com")
        .path_and_query(format!("/login/oauth/authorize?client_id={client_id}&redirect_uri={redirect_uri}&state={state}"))
        .build().into_diagnostic()
}

pub(crate) fn github_redirect_uri(config: &AppState) -> String {
    format!("{}/github_oauth", config.app.base_url)
}
