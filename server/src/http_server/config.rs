use axum::extract::FromRef;

use crate::{twitch::TwitchConfig, AppConfig, Config};

impl FromRef<Config> for TwitchConfig {
    fn from_ref(config: &Config) -> Self {
        config.twitch.clone()
    }
}

impl FromRef<Config> for AppConfig {
    fn from_ref(config: &Config) -> Self {
        config.app.clone()
    }
}
