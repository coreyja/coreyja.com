use axum::{
    extract::{Path, State},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router, Server,
};

use std::{net::SocketAddr, sync::Arc};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};

use crate::{
    http_server::image_optimization::Assets,
    posts::blog::{BlogPosts, ToCanonicalPath},
    AppState,
};
pub use config::*;
use errors::*;

use self::image_optimization::CACHE_DIR;

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

pub(crate) mod image_optimization;

const TAILWIND_STYLES: &str = include_str!("../../../target/tailwind.css");

type ResponseResult<T = Response> = Result<T, MietteError>;

pub(crate) async fn run_axum(config: AppState) -> miette::Result<()> {
    let syntax_css = syntect::html::css_for_theme_with_class_style(
        &config.markdown_to_html_context.theme,
        syntect::html::ClassStyle::Spaced,
    )
    .unwrap();

    let app = Router::new()
        .route("/static/*path", get(static_asset_route))
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
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_response(DefaultOnResponse::new().include_headers(true)),
        );

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

async fn static_asset_route(
    Path(p): Path<String>,
    State(assets): State<Arc<Assets<'static>>>,
) -> ResponseResult {
    let path = p.strip_prefix('/').unwrap_or(&p);
    let path = path.strip_suffix('/').unwrap_or(path);

    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let asset = assets.get(path);

    let Some(asset) = asset else {
        return Ok(
            (
                axum::http::StatusCode::NOT_FOUND,
                format!("Static asset {} not found", path)
            )
        .into_response());
    };

    match asset {
        image_optimization::Asset::Image(i) => {
            let mut headers = axum::http::HeaderMap::new();
            headers.insert(
                axum::http::header::CONTENT_TYPE,
                "image/png".parse().unwrap(),
            );

            let image = cacache::read_hash(CACHE_DIR, &i.resized_hash)
                .await
                .unwrap();

            Ok((headers, image).into_response())
        },
        image_optimization::Asset::Other(f) => {
            let mut headers = axum::http::HeaderMap::new();
            headers.insert(
                axum::http::header::CONTENT_TYPE,
                mime.to_string().parse().unwrap(),
            );

            Ok((headers, f.contents().to_vec()).into_response())
        }
    }

    // let image = cacache::read_hash(CACHE_DIR, &asset.op).unwrap();

    // let mut buffer = BufWriter::new(Cursor::new(Vec::new()));
    // image.write_to(&mut buffer, ImageFormat::Png).unwrap();

    // let mut headers = axum::http::HeaderMap::new();
    // headers.insert(
    //     axum::http::header::CONTENT_TYPE,
    //     mime.to_string().parse().unwrap(),
    // );

    // (headers, buffer.into_inner().unwrap().into_inner()).into_response())
}

async fn newsletter_get() -> ResponseResult {
    Ok((
        axum::http::StatusCode::OK,
        templates::newsletter::newsletter_page(),
    )
        .into_response())
}
