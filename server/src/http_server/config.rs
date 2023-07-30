use std::sync::Arc;

use axum::extract::FromRef;

use crate::{
    posts::{blog::BlogPosts, til::TilPosts},
    twitch::TwitchConfig,
    AppConfig, AppState,
};

use super::pages::blog::md::SyntaxHighlightingContext;

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

impl FromRef<AppState> for SyntaxHighlightingContext {
    fn from_ref(config: &AppState) -> Self {
        config.markdown_to_html_context.clone()
    }
}

impl FromRef<AppState> for Arc<BlogPosts> {
    fn from_ref(config: &AppState) -> Self {
        config.blog_posts.clone()
    }
}

impl FromRef<AppState> for Arc<TilPosts> {
    fn from_ref(config: &AppState) -> Self {
        config.til_posts.clone()
    }
}
