use std::{path::PathBuf, str::FromStr};

use posts::blog::BlogPosts;
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

use super::{
    admin, api, auth, get, pages, post, templates, webhooks, AppState, Arc, IntoResponse, Path,
    Redirect, Response, ResponseResult, Result, Router, ServerError, State, ToCanonicalPath, Uri,
    COMIC_CODE_STYLES, STATIC_ASSETS, TAILWIND_STYLES,
};

pub(crate) fn make_router(syntax_css: String) -> Router<AppState> {
    Router::new()
        .route("/_", get(pages::admin::versions))
        .route("/static/{*path}", get(static_assets))
        .route("/styles/syntax.css", get(|| async move { syntax_css }))
        .route("/styles/tailwind.css", get(|| async { TAILWIND_STYLES }))
        .route(
            "/styles/comic_code.css",
            get(|| async { COMIC_CODE_STYLES }),
        )
        .route("/", get(pages::home::home_page))
        .route("/privacy", get(pages::legal::privacy_policy))
        .route("/contact", get(pages::contact::contact))
        .route("/posts/rss.xml", get(pages::blog::rss_feed))
        .route("/rss.xml", get(pages::blog::full_rss_feed))
        .route("/posts", get(pages::blog::posts_index))
        .route(
            "/posts/weekly/",
            // I accidentally published by first newsletter under this path
            // so I'm redirecting it to the newsletter home page. I'll
            // update the few links I made outside this blog to the correct link
            get(|| async { Redirect::permanent("/newsletter") }),
        )
        .route("/posts/{*key}", get(pages::blog::post_get))
        .route("/til", get(pages::til::til_index))
        .route("/til/rss.xml", get(pages::til::rss_feed))
        .route("/til/{slug}", get(pages::til::til_get))
        .route("/projects", get(pages::projects::projects_index))
        .route("/projects/{slug}", get(pages::projects::projects_get))
        .route("/videos", get(pages::videos::video_index))
        .route("/videos/{id}", get(pages::videos::video_get))
        .route("/tags/{*tag}", get(redirect_to_posts_index))
        .route("/year/{*year}", get(redirect_to_posts_index))
        .nest("/blog", old_blog_routes())
        .route("/newsletter", get(newsletter_get))
        .route("/auth/github", get(auth::github_oauth::github_oauth))
        .nest("/login", pages::login::routes())
        .route("/my/account", get(pages::account::account_page))
        .route("/my/sponsorship", get(pages::account::sponsorship_page))
        .route("/admin/auth/google", get(admin::auth::google_auth))
        .route(
            "/admin/auth/google/callback",
            get(admin::auth::google_auth_callback),
        )
        .route(
            "/admin/jobs/refresh_youtube",
            post(admin::job_routes::refresh_youtube),
        )
        .route("/admin", get(admin::dashboard))
        .route("/admin/crons", get(admin::crons::list_crons))
        .route("/admin/crons/reset", post(admin::crons::reset_cron))
        .route("/admin/crons/run", post(admin::crons::run_cron))
        .route("/admin/threads", get(admin::threads::threads_app))
        .route(
            "/admin/threads/*path",
            get(admin::threads::serve_thread_assets),
        )
        .route("/admin/api/threads", get(api::threads::list_threads))
        .route("/admin/api/threads", post(api::threads::create_thread))
        .route("/admin/api/threads/:id", get(api::threads::get_thread))
        .route("/webhooks/cookd", post(webhooks::cookd::handler))
        .nest("/api", api_routes())
        .route("/bytes", get(pages::bytes::bytes_index))
        .route("/bytes/{slug}", get(pages::bytes::byte_get))
        .route(
            "/bytes/{slug}/leaderboard",
            get(pages::bytes::single_leaderboard),
        )
        .route("/bytes_leaderboard", get(pages::bytes::overall_leaderboard))
        .fallback(fallback)
}

fn old_blog_routes() -> Router<AppState> {
    #[derive(Serialize, Deserialize)]
    struct OldRoutePath {
        year: String,
        month: String,
        date: String,
        slug: String,
    }

    Router::new()
        .route(
            "/{year}/{month}/{date}/{slug}",
            get(
                |Path(OldRoutePath { slug, .. }): Path<OldRoutePath>| async move {
                    let Ok(mut slug) = PathBuf::from_str(&slug);

                    slug.set_extension("");

                    Redirect::permanent(&format!("/posts/{}", slug.display())).into_response()
                },
            ),
        )
        .fallback(redirect_to_posts_index)
}

async fn redirect_to_posts_index() -> impl IntoResponse {
    Redirect::permanent("/posts")
}

async fn fallback(uri: Uri, State(posts): State<Arc<BlogPosts>>) -> Result<Response, ServerError> {
    let path = uri.path();
    let decoded = urlencoding::decode(path)?;
    let key = decoded.as_ref();
    let key = key.strip_prefix('/').unwrap_or(key);
    let key = key.strip_suffix('/').unwrap_or(key);

    let post = posts.posts().iter().find(|p| p.matches_path(key).is_some());

    let resp = match post {
        Some(post) => {
            Redirect::permanent(&format!("/posts/{}", post.path.canonical_path())).into_response()
        }
        None => axum::http::StatusCode::NOT_FOUND.into_response(),
    };

    Ok(resp)
}

async fn static_assets(Path(p): Path<String>) -> ResponseResult {
    let path = p.strip_prefix('/').unwrap_or(&p);
    let path = path.strip_suffix('/').unwrap_or(path);

    let entry = STATIC_ASSETS.get_file(path);

    let Some(entry) = entry else {
        return Ok((
            axum::http::StatusCode::NOT_FOUND,
            format!("Static asset {path} not found"),
        )
            .into_response());
    };

    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(axum::http::header::CONTENT_TYPE, mime.to_string().parse()?);

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

fn api_routes() -> Router<AppState> {
    Router::new()
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
}
