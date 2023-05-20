use axum::extract::FromRef;

use crate::{twitch::TwitchConfig, AppConfig, AppState};

use super::pages::blog::md::HtmlRenderContext;

impl FromRef<AppState> for TwitchConfig {
    fn from_ref(config: &AppState) -> Self {
        config.twitch.clone()
    }
}

impl FromRef<AppState> for AppConfig {
    fn from_ref(config: &AppState) -> Self {
        config.app.clone()
    }
}

impl FromRef<AppState> for HtmlRenderContext {
    fn from_ref(config: &AppState) -> Self {
        config.markdown_to_html_context.clone()
    }
}
