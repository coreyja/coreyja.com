use cja::color_eyre::eyre::Context;
use serde::{Deserialize, Serialize};

pub mod graphql;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinearConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub webhook_secret: String,
}

impl LinearConfig {
    pub fn from_env() -> cja::Result<Self> {
        let client_id = std::env::var("LINEAR_CLIENT_ID")
            .wrap_err("Missing LINEAR_CLIENT_ID environment variable")?;
        let client_secret = std::env::var("LINEAR_CLIENT_SECRET")
            .wrap_err("Missing LINEAR_CLIENT_SECRET environment variable")?;
        let redirect_uri = std::env::var("LINEAR_REDIRECT_URI")
            .wrap_err("Missing LINEAR_REDIRECT_URI environment variable")?;
        let webhook_secret = std::env::var("LINEAR_WEBHOOK_SECRET")
            .wrap_err("Missing LINEAR_WEBHOOK_SECRET environment variable")?;

        Ok(Self {
            client_id,
            client_secret,
            redirect_uri,
            webhook_secret,
        })
    }
}
