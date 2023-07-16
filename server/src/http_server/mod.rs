use axum::{
    extract::{Path, State},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router, Server,
};
use include_dir::*;
use miette::{Context, IntoDiagnostic};
use std::{net::SocketAddr, sync::Arc};
use tower_http::trace::TraceLayer;

use crate::{
    posts::blog::{BlogPosts, ToCanonicalPath},
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
mod routes;
mod server_tracing;
mod templates;

const TAILWIND_STYLES: &str = include_str!("../../../target/tailwind.css");

const STATIC_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/static");

type ResponseResult<T = Response> = Result<T, MietteError>;

pub(crate) async fn run_axum(config: AppState) -> miette::Result<()> {
    let syntax_css = syntect::html::css_for_theme_with_class_style(
        &config.markdown_to_html_context.theme,
        syntect::html::ClassStyle::Spaced,
    )
    .unwrap();

    let tracer = server_tracing::Tracer;
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(tracer)
        .on_response(tracer);

    let app = routes::make_router(syntax_css)
        .with_state(config)
        .layer(trace_layer);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .into_diagnostic()
        .wrap_err("Failed to run server")
}
