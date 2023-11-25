use crate::*;

use miette::Result;

#[derive(Debug, Clone)]
pub(crate) struct GithubConfig {
    pub(crate) app_id: u64,
    pub(crate) client_id: String,
    pub(crate) client_secret: String,
}

impl GithubConfig {
    #[instrument]
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
