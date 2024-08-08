use std::sync::Arc;

use axum::extract::FromRef;
use posts::{blog::BlogPosts, projects::Projects, til::TilPosts};

use crate::{google::GoogleConfig, twitch::TwitchConfig, AppConfig, AppState};

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
        config.syntax_highlighting_context.clone()
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

impl FromRef<AppState> for Arc<Projects> {
    fn from_ref(config: &AppState) -> Self {
        config.projects.clone()
    }
}

impl FromRef<AppState> for GoogleConfig {
    fn from_ref(config: &AppState) -> Self {
        config.google.clone()
    }
}
