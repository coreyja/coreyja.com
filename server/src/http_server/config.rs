use axum::extract::FromRef;

use crate::{twitch::TwitchConfig, AppConfig, Config};

use super::pages::blog::md::HtmlRenderContext;

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

impl FromRef<Config> for HtmlRenderContext {
    fn from_ref(config: &Config) -> Self {
        config.markdown_to_html_context.clone()
    }
}
