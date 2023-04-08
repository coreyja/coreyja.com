use crate::{
    http_server::{
        api::external::{github_oauth, twitch_oauth},
        pages::home::home_page,
    },
    *,
};

use axum::{
    extract::FromRef,
    response::IntoResponse,
    routing::{get, post},
    Router, Server,
};

mod admin;
mod blog;
mod templates;

mod pages {
    pub mod home;
}

mod api {
    pub mod external {
        pub mod github_oauth;
        pub mod twitch_oauth;
    }
}

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

const TAILWIND_STYLES: &str = include_str!("../../../target/tailwind.css");

pub(crate) async fn run_axum(config: Config) -> color_eyre::Result<()> {
    let app = Router::new()
        .route("/styles/tailwind.css", get(|| async { TAILWIND_STYLES }))
        .route("/", get(home_page))
        .route("/twitch_oauth", get(twitch_oauth::handler))
        .route("/github_oauth", get(github_oauth::handler))
        .route(
            "/admin/upwork/proposals/:id",
            get(admin::upwork_proposal_get),
        )
        .route(
            "/admin/upwork/proposals/:id",
            post(admin::upwork_proposal_post),
        )
        .route("/posts", get(blog::posts_index))
        .route("/posts/*key", get(blog::post_get))
        .with_state(config);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}

pub struct EyreError(color_eyre::Report);

impl IntoResponse for EyreError {
    fn into_response(self) -> axum::response::Response {
        self.0.to_string().into_response()
    }
}

impl<T> From<T> for EyreError
where
    T: Into<color_eyre::Report>,
{
    fn from(err: T) -> Self {
        EyreError(err.into())
    }
}
