use axum::{
    body::Body,
    http::{Request, Response},
    Router,
};
use serde::de::DeserializeOwned;
use sqlx::PgPool;
use std::sync::Arc;

use crate::http_server::pages::blog::md::SyntaxHighlightingContext;
use crate::{AppConfig, AppState};

pub async fn create_test_app(pool: PgPool) -> Router {
    // Set minimal environment variables for testing
    std::env::set_var("APP_BASE_URL", "http://localhost:3000");
    std::env::set_var("TWITCH_CLIENT_ID", "test-client-id");
    std::env::set_var("TWITCH_CLIENT_SECRET", "test-client-secret");
    std::env::set_var("GITHUB_CLIENT_ID", "test-github-id");
    std::env::set_var("GITHUB_CLIENT_SECRET", "test-github-secret");
    std::env::set_var("OPENAI_API_KEY", "test-openai-key");
    std::env::set_var("COOKIE_KEY", "test-cookie-key-32-bytes-long!!!!");
    std::env::set_var("ENCRYPT_KEY", "test-encrypt-key-32-bytes-long!!");
    std::env::set_var("LINEAR_CLIENT_ID", "test-linear-id");
    std::env::set_var("LINEAR_CLIENT_SECRET", "test-linear-secret");
    std::env::set_var("LINEAR_WEBHOOK_SECRET", "test-webhook-secret");

    let discord_setup = crate::discord::setup().await.unwrap();
    let discord_client = discord_setup.map(|d| d.client);

    // Create a minimal test state
    let state = AppState {
        twitch: crate::twitch::TwitchConfig::from_env().unwrap(),
        github: crate::github::GithubConfig::from_env().unwrap(),
        open_ai: openai::OpenAiConfig::from_env().unwrap(),
        google: crate::google::GoogleConfig::from_env().unwrap(),
        linear: crate::linear::LinearConfig::from_env().unwrap(),
        app: AppConfig::from_env().unwrap(),
        standup: crate::state::StandupConfig::from_env().unwrap(),
        syntax_highlighting_context: SyntaxHighlightingContext::default(),
        blog_posts: Arc::new(posts::blog::BlogPosts::from_static_dir().unwrap()),
        til_posts: Arc::new(posts::til::TilPosts::from_static_dir().unwrap()),
        projects: Arc::new(posts::projects::Projects::from_static_dir().unwrap()),
        versions: crate::state::VersionInfo {
            git_commit: "test-commit",
            rustc_version: "test-rustc",
        },
        db: pool,
        cookie_key: cja::server::cookies::CookieKey::from_env_or_generate().unwrap(),
        encrypt_config: crate::encrypt::Config::from_env().unwrap(),
        posthog_key: None,
        discord: discord_client,
    };

    let syntax_css = String::new(); // Empty for tests
    crate::http_server::routes::make_router(syntax_css).with_state(state)
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
