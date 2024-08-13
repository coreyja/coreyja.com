use std::sync::Arc;

use cja::{color_eyre::eyre::Context, server::cookies::CookieKey};
use db::setup_db_pool;
use openai::OpenAiConfig;
use posts::{blog::BlogPosts, projects::Projects, til::TilPosts};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::instrument;
use url::Url;

use crate::{
    encrypt, github::GithubConfig, google::GoogleConfig,
    http_server::pages::blog::md::SyntaxHighlightingContext, twitch::TwitchConfig,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub base_url: Url,
    pub imgproxy_url: Option<String>,
}

impl AppConfig {
    #[instrument(name = "AppConfig::from_env")]
    pub fn from_env() -> cja::Result<Self> {
        let base_url = std::env::var("APP_BASE_URL")
            .wrap_err("Missing APP_BASE_URL, needed for app launch")?;
        let base_url = Url::parse(&base_url).wrap_err("Invalid APP_BASE_URL not parsable")?;
        Ok(Self {
            base_url,
            imgproxy_url: std::env::var("IMGPROXY_URL").ok(),
        })
    }

    pub fn app_url(&self, path: &str) -> String {
        let mut url = self.base_url.clone();

        url.set_path(path);

        url.into()
    }

    pub fn home_page(&self) -> String {
        self.base_url.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub git_commit: &'static str,
    pub rustc_version: &'static str,
}

impl VersionInfo {
    #[instrument(name = "VersionInfo::from_env")]
    fn from_env() -> Self {
        Self {
            git_commit: env!("VERGEN_GIT_SHA"),
            rustc_version: env!("VERGEN_RUSTC_SEMVER"),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    pub twitch: TwitchConfig,
    pub github: GithubConfig,
    pub open_ai: OpenAiConfig,
    pub google: GoogleConfig,
    pub app: AppConfig,
    pub syntax_highlighting_context: SyntaxHighlightingContext,
    pub blog_posts: Arc<BlogPosts>,
    pub til_posts: Arc<TilPosts>,
    pub projects: Arc<Projects>,
    pub versions: VersionInfo,
    pub db: PgPool,
    pub cookie_key: CookieKey,
    pub encrypt_config: encrypt::Config,
    pub posthog_key: Option<String>,
}

impl AppState {
    #[instrument(name = "AppState::from_env", err)]
    pub async fn from_env() -> cja::Result<Self> {
        let blog_posts = BlogPosts::from_static_dir()?;
        let blog_posts = Arc::new(blog_posts);

        let til_posts = TilPosts::from_static_dir()?;
        let til_posts = Arc::new(til_posts);

        let projects = Projects::from_static_dir()?;
        let projects = Arc::new(projects);

        let cookie_key = CookieKey::from_env_or_generate()?;

        let app_state = AppState {
            twitch: TwitchConfig::from_env()?,
            github: GithubConfig::from_env()?,
            app: AppConfig::from_env()?,
            open_ai: OpenAiConfig::from_env()?,
            google: GoogleConfig::from_env()?,
            syntax_highlighting_context: SyntaxHighlightingContext::default(),
            versions: VersionInfo::from_env(),
            blog_posts,
            til_posts,
            projects,
            db: setup_db_pool().await?,
            cookie_key,
            encrypt_config: encrypt::Config::from_env()?,
            posthog_key: std::env::var("POSTHOG_KEY").ok(),
        };

        Ok(app_state)
    }
}

impl cja::app_state::AppState for AppState {
    fn version(&self) -> &str {
        self.versions.git_commit
    }

    fn db(&self) -> &sqlx::PgPool {
        &self.db
    }

    fn cookie_key(&self) -> &CookieKey {
        &self.cookie_key
    }
}
