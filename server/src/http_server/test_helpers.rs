//! Shared test helpers for building a real `make_router()` against a fully
//! populated `AppState`. Designed for unit tests that need to exercise actual
//! route wiring (slug matching, extractor binding, response shape) without
//! starting a server or hitting external services.
//!
//! Discord is built via [`crate::discord::DiscordClient::for_testing`], which
//! constructs `serenity::Http` and `serenity::Cache` directly — no gateway
//! connection, no network. The Postgres pool is wired lazily by default
//! ([`lazy_test_pool`]); tests that actually exercise the DB should accept a
//! `PgPool` from `#[sqlx::test(migrations = "../db/migrations")]` and pass it
//! to [`create_test_app_with_pool`].

use axum::{
    body::Body,
    http::{Request, Response},
    Router,
};
use serde::de::DeserializeOwned;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use std::sync::Arc;

use crate::http_server::pages::blog::md::SyntaxHighlightingContext;
use crate::{AppConfig, AppState};

/// Build a real `Router<AppState>` populated through `make_router()` with a
/// lazy (never-connecting) Postgres pool. Suitable for tests that don't touch
/// the database. Tests that need real DB should call
/// [`create_test_app_with_pool`] from a `#[sqlx::test]` fn instead.
pub async fn create_test_app() -> Router {
    create_test_app_with_pool(lazy_test_pool()).await
}

/// Like [`create_test_app`] but uses the caller-supplied `PgPool`. Use this
/// from `#[sqlx::test(migrations = "../db/migrations")]` tests.
pub async fn create_test_app_with_pool(pool: PgPool) -> Router {
    set_test_env_vars();

    let state = AppState {
        twitch: crate::twitch::TwitchConfig::from_env().unwrap(),
        github: crate::github::GithubConfig::from_env().unwrap(),
        open_ai: openai::OpenAiConfig::from_env().unwrap(),
        google: crate::google::GoogleConfig::from_env().unwrap(),
        linear: crate::linear::LinearConfig::from_env().unwrap(),
        anthropic: crate::anthropic::AnthropicConfig::from_env().unwrap(),
        app: AppConfig::from_env().unwrap(),
        syntax_highlighting_context: SyntaxHighlightingContext,
        blog_posts: Arc::new(posts::blog::BlogPosts::from_static_dir().unwrap()),
        note_posts: Arc::new(posts::notes::NotePosts::from_static_dir().unwrap()),
        podcast_episodes: Arc::new(posts::podcast::PodcastEpisodes::from_static_dir().unwrap()),
        projects: Arc::new(posts::projects::Projects::from_static_dir().unwrap()),
        versions: crate::state::VersionInfo {
            git_commit: "test-commit",
            rustc_version: "test-rustc",
        },
        db: pool,
        cookie_key: cja::server::cookies::CookieKey::from_env_or_generate().unwrap(),
        encrypt_config: crate::encrypt::Config::from_env().unwrap(),
        posthog_key: None,
        discord: crate::discord::DiscordClient::for_testing(),
    };

    crate::http_server::routes::make_router().with_state(state)
}

/// A `PgPool` that never actually connects. `connect_lazy_with` defers
/// connection establishment until the first query, so handlers that don't
/// touch the DB run fine against this pool.
fn lazy_test_pool() -> PgPool {
    PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy_with(PgConnectOptions::new())
}

/// Set every env var that any `*::from_env()` call in `AppState` construction
/// reads. Values are obviously-fake placeholders — never use this helper in a
/// real binary.
fn set_test_env_vars() {
    let vars: &[(&str, &str)] = &[
        ("APP_BASE_URL", "http://localhost:3000"),
        ("TWITCH_CLIENT_ID", "test-twitch-client-id"),
        ("TWITCH_CLIENT_SECRET", "test-twitch-client-secret"),
        ("TWITCH_BOT_USER_ID", "test-twitch-bot-user-id"),
        ("TWITCH_CHANNEL_USER_ID", "test-twitch-channel-user-id"),
        ("GITHUB_APP_ID", "1"),
        ("GITHUB_APP_CLIENT_ID", "test-github-client-id"),
        ("GITHUB_APP_CLIENT_SECRET", "test-github-client-secret"),
        ("GITHUB_PERSONAL_ACCESS_TOKEN", "test-github-pat"),
        ("GITHUB_APP_PRIVATE_KEY", "test-github-app-private-key"),
        ("GOOGLE_CLIENT_ID", "test-google-client-id"),
        ("GOOGLE_CLIENT_SECRET", "test-google-client-secret"),
        ("OPEN_AI_API_KEY", "test-openai-key"),
        ("ANTHROPIC_API_KEY", "test-anthropic-key"),
        ("LINEAR_CLIENT_ID", "test-linear-id"),
        ("LINEAR_CLIENT_SECRET", "test-linear-secret"),
        ("LINEAR_WEBHOOK_SECRET", "test-linear-webhook-secret"),
        ("ENCRYPTION_SECRET_KEY", "test-encryption-secret-key"),
    ];
    for (k, v) in vars {
        if std::env::var_os(k).is_none() {
            std::env::set_var(k, v);
        }
    }
}

pub fn admin_request_builder() -> axum::http::request::Builder {
    // In tests, we bypass real authentication
    // The test must modify the admin route handler to accept test requests
    Request::builder().header("x-test-mode", "admin")
}

pub async fn response_body_json<T: DeserializeOwned>(response: Response<Body>) -> T {
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body_bytes).unwrap()
}
