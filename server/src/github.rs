use axum::http::Uri;
use color_eyre::Result;

#[derive(Debug, Clone)]
pub(crate) struct GithubConfig {
    pub(crate) app_id: u64,
    pub(crate) client_id: String,
    pub(crate) client_secret: String,
    pub(crate) redirect_uri: String,
}

impl GithubConfig {
    pub(crate) fn from_env() -> Result<Self> {
        Ok(Self {
            app_id: std::env::var("GITHUB_APP_ID")?.parse()?,
            client_id: std::env::var("GITHUB_APP_CLIENT_ID")?,
            client_secret: std::env::var("GITHUB_APP_CLIENT_SECRET")?,
            redirect_uri: std::env::var("GITHUB_APP_REDIRECT_URI")?,
        })
    }
}

pub(crate) fn generate_user_github_link(config: &GithubConfig, state: &str) -> Result<Uri> {
    let client_id = &config.client_id;
    let redirect_uri = &config.redirect_uri;

    Ok(Uri::builder()
        .scheme("https")
        .authority("github.com")
        .path_and_query(format!("/login/oauth/authorize?client_id={client_id}&redirect_uri={redirect_uri}&state={state}"))
        .build()?)
}
