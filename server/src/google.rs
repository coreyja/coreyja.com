use miette::{Context, IntoDiagnostic};

#[derive(Debug, Clone)]
pub struct GoogleConfig {
    pub client_id: String,
    pub client_secret: String,
}

impl GoogleConfig {
    pub fn from_env() -> miette::Result<Self> {
        Ok(Self {
            client_id: std::env::var("GOOGLE_CLIENT_ID")
                .into_diagnostic()
                .context("GOOGLE_CLIENT_ID env var missing")?,
            client_secret: std::env::var("GOOGLE_CLIENT_SECRET")
                .into_diagnostic()
                .context("GOOGLE_CLIENT_SECRET env var missing")?,
        })
    }
}
