use axum::{
    extract::{Path, State},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router, Server,
};
use include_dir::*;
use std::{net::SocketAddr, sync::Arc};
use tower_http::trace::{MakeSpan, OnResponse, TraceLayer};
use tracing::Level;

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

    let app = Router::new()
        .route("/static/*path", get(static_assets))
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
        .route(
            "/rss.xml",
            get(|| async { Redirect::permanent("/posts/rss.xml") }),
        )
        .route("/posts", get(pages::blog::posts_index))
        .route("/posts/*key", get(pages::blog::post_get))
        .route("/til", get(pages::til::til_index))
        .route("/til/:slug", get(pages::til::til_get))
        .route("/tags/*tag", get(redirect_to_posts_index))
        .route("/year/*year", get(redirect_to_posts_index))
        .route("/newsletter", get(newsletter_get))
        .fallback(fallback)
        .with_state(config)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(MyMakeSpan)
                .on_response(MyMakeSpan),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|_| miette::miette!("Failed to run server"))?;

    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct MyMakeSpan;

impl<Body> MakeSpan<Body> for MyMakeSpan {
    fn make_span(&mut self, request: &axum::http::Request<Body>) -> tracing::Span {
        tracing::span!(
            Level::INFO,
            "request",
            kind = "server",
            uri = %request.uri(),
            ulr.path = %request.uri().path(),
            url.query = request.uri().query(),
            url.scheme = request.uri().scheme_str(),
            server.address = request.uri().host(),
            server.port = request.uri().port_u16(),
            http_version = ?request.version(),
            user_agent.original = request.headers().get("user-agent").and_then(|h| h.to_str().ok()),
            http.request.method = %request.method(),
            http.request.header.host = request.headers().get("host").and_then(|h| h.to_str().ok()),
            http.request.header.forwarded_for = request.headers().get("x-forwarded-for").and_then(|h| h.to_str().ok()),
            http.request.header.forwarded_proto = request.headers().get("x-forwarded-proto").and_then(|h| h.to_str().ok()),
            http.request.header.host = request.headers().get("x-forwarded-ssl").and_then(|h| h.to_str().ok()),
            http.request.header.referer = request.headers().get("referer").and_then(|h| h.to_str().ok()),
            http.request.header.fly_forwarded_port = request.headers().get("fly-forwarded-port").and_then(|h| h.to_str().ok()),
            http.request.header.fly_region = request.headers().get("fly-region").and_then(|h| h.to_str().ok()),
            http.request.header.via = request.headers().get("via").and_then(|h| h.to_str().ok()),

            http.response.status_code = tracing::field::Empty,
            http.response.header.content_type = tracing::field::Empty,
        )
    }
}

impl<Body> OnResponse<Body> for MyMakeSpan {
    fn on_response(
        self,
        response: &axum::http::Response<Body>,
        latency: std::time::Duration,
        span: &tracing::Span,
    ) {
        let status_code = response.status().as_u16();
        tracing::event!(
            Level::INFO,
            status = status_code,
            latency = format_args!("{} ms", latency.as_millis()),
            "finished processing request"
        );

        span.record("http.response.status_code", status_code);
        span.record(
            "http.response.header.content_type",
            response
                .headers()
                .get("content-type")
                .and_then(|h| h.to_str().ok()),
        );
    }
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

async fn static_assets(Path(p): Path<String>) -> ResponseResult {
    let path = p.strip_prefix('/').unwrap_or(&p);
    let path = path.strip_suffix('/').unwrap_or(path);

    let entry = STATIC_ASSETS.get_file(path);

    let Some(entry) = entry else {
        return Ok(
            (
                axum::http::StatusCode::NOT_FOUND,
                format!("Static asset {} not found", path)
            )
        .into_response());
    };

    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        mime.to_string().parse().unwrap(),
    );

    Ok((headers, entry.contents()).into_response())
}

async fn newsletter_get(State(posts): State<Arc<BlogPosts>>) -> ResponseResult {
    let newsletters = posts
        .by_recency()
        .into_iter()
        .filter(|p| p.frontmatter.is_newsletter)
        .collect::<Vec<_>>();

    Ok((
        axum::http::StatusCode::OK,
        templates::newsletter::newsletter_page(newsletters),
    )
        .into_response())
}
