use axum::{
    extract::State,
    http::Uri,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router, Server,
};
use std::{net::SocketAddr, sync::Arc};
use tower_http::trace::TraceLayer;

use crate::{
    blog::{BlogPosts, ToCanonicalPath},
    AppState,
};
pub use config::*;
use errors::*;

pub(crate) mod cmd;

pub(crate) mod pages {
    pub mod admin;
    pub mod blog;
    pub mod home;
    pub mod til;
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

pub(crate) async fn run_axum(config: AppState) -> miette::Result<()> {
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
        .route("/til", get(pages::til::til_index))
        .route("/tags/*tag", get(redirect_to_posts_index))
        .route("/year/*year", get(redirect_to_posts_index))
        .fallback(fallback)
        .with_state(config)
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|_| miette::miette!("Failed to run server"))?;

    Ok(())
}

async fn redirect_to_posts_index() -> impl IntoResponse {
    Redirect::permanent("/posts")
}

async fn fallback(uri: Uri, State(posts): State<Arc<BlogPosts>>) -> Response {
    let path = uri.path();
    let decoded = urlencoding::decode(path).unwrap();
    let key = decoded.as_ref();
    let key = key.strip_prefix('/').unwrap_or(key);
    let key = key.strip_suffix('/').unwrap_or(key);

    let post = posts.posts().iter().find(|p| p.matches_path(key).is_some());

    match post {
        Some(post) => {
            Redirect::permanent(&format!("/posts/{}", post.canonical_path())).into_response()
        }
        None => axum::http::StatusCode::NOT_FOUND.into_response(),
    }
}
