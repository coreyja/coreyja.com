use axum::{
    routing::{get, post},
    Router, Server,
};
use std::net::SocketAddr;

use crate::Config;
pub use config::*;
use errors::*;

pub(crate) mod cmd;

pub(crate) mod pages {
    pub mod admin;
    pub mod blog;
    pub mod home;
}

mod api {
    pub mod external {
        pub mod github_oauth;
        pub mod twitch_oauth;
    }
}

mod config;
pub mod errors;
mod templates;

const TAILWIND_STYLES: &str = include_str!("../../../target/tailwind.css");

pub(crate) async fn run_axum(config: Config) -> miette::Result<()> {
    let syntax_css = syntect::html::css_for_theme_with_class_style(
        &config.markdown_to_html_context.theme,
        syntect::html::ClassStyle::Spaced,
    )
    .unwrap();

    let app = Router::new()
        .route("/styles/syntax.css", get(|| async move { syntax_css }))
        .route("/styles/tailwind.css", get(|| async { TAILWIND_STYLES }))
        .route("/", get(pages::home::home_page))
        .route("/twitch_oauth", get(api::external::twitch_oauth::handler))
        .route("/github_oauth", get(api::external::github_oauth::handler))
        .route(
            "/admin/upwork/proposals/:id",
            get(pages::admin::upwork_proposal_get),
        )
        .route(
            "/admin/upwork/proposals/:id",
            post(pages::admin::upwork_proposal_post),
        )
        .route("/posts/rss.xml", get(pages::blog::rss_feed))
        .route("/posts", get(pages::blog::posts_index))
        .route("/posts/*key", get(pages::blog::post_get))
        .with_state(config);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|_| miette::miette!("Failed to run server"))?;

    Ok(())
}
