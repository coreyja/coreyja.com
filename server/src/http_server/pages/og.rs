//! OG card SVG endpoints. These return raw SVG; `imgproxy` is responsible for rasterizing
//! and caching the PNG version that social scrapers actually consume.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use posts::{blog::BlogPosts, podcast::PodcastEpisodes};

use crate::http_server::templates::og::{
    fetch_youtube_thumbnail_b64, render_card_svg, CardData, CardTag,
};

const SVG_CACHE_CONTROL: &str = "public, max-age=86400, stale-while-revalidate=604800";

fn svg_response(svg: String) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "image/svg+xml".parse().unwrap());
    headers.insert(header::CACHE_CONTROL, SVG_CACHE_CONTROL.parse().unwrap());
    (headers, svg).into_response()
}

pub async fn og_post_svg(
    State(posts): State<Arc<BlogPosts>>,
    Path(slug): Path<String>,
) -> Result<Response, StatusCode> {
    let slug = slug.strip_suffix(".svg").unwrap_or(&slug);
    let post = posts
        .posts()
        .iter()
        .find(|p| !p.frontmatter.is_newsletter && p.og_slug() == slug)
        .ok_or(StatusCode::NOT_FOUND)?;
    let data = CardData {
        title: &post.frontmatter.title,
        date: post.frontmatter.date,
        tag: CardTag::Posts,
        youtube_thumbnail_b64: None,
    };
    Ok(svg_response(render_card_svg(&data)))
}

pub async fn og_weekly_svg(
    State(posts): State<Arc<BlogPosts>>,
    Path(slug): Path<String>,
) -> Result<Response, StatusCode> {
    let slug = slug.strip_suffix(".svg").unwrap_or(&slug);
    let post = posts
        .posts()
        .iter()
        .find(|p| p.frontmatter.is_newsletter && p.og_slug() == slug)
        .ok_or(StatusCode::NOT_FOUND)?;
    let data = CardData {
        title: &post.frontmatter.title,
        date: post.frontmatter.date,
        tag: CardTag::Newsletter,
        youtube_thumbnail_b64: None,
    };
    Ok(svg_response(render_card_svg(&data)))
}

pub async fn og_podcast_svg(
    State(episodes): State<Arc<PodcastEpisodes>>,
    Path(slug): Path<String>,
) -> Result<Response, StatusCode> {
    let slug = slug.strip_suffix(".svg").unwrap_or(&slug);
    let ep = episodes
        .episodes
        .iter()
        .find(|e| e.frontmatter.slug == slug)
        .ok_or(StatusCode::NOT_FOUND)?;
    let thumbnail = fetch_youtube_thumbnail_b64(&ep.frontmatter.youtube_id).await;
    let data = CardData {
        title: &ep.frontmatter.title,
        date: ep.frontmatter.date,
        tag: CardTag::Podcast,
        youtube_thumbnail_b64: thumbnail,
    };
    Ok(svg_response(render_card_svg(&data)))
}

#[cfg(test)]
mod tests {
    //! NOTE: These tests deliberately build a minimal Axum router with only the OG routes
    //! rather than going through `test_helpers::create_test_app`. The shared helper depends
    //! on a complete `AppState`, which requires Discord/Twitch/Google/etc. env vars that
    //! aren't available in unit-test runs. Since the OG handlers only need `Arc<BlogPosts>`
    //! and `Arc<PodcastEpisodes>`, a focused mini-router is sufficient to verify routing
    //! and slug gating.
    use super::{og_podcast_svg, og_post_svg, og_weekly_svg};
    use axum::{
        body::Body,
        extract::FromRef,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use posts::{blog::BlogPosts, podcast::PodcastEpisodes};
    use std::sync::Arc;
    use tower::ServiceExt;

    #[derive(Clone)]
    struct TestState {
        blog: Arc<BlogPosts>,
        podcast: Arc<PodcastEpisodes>,
    }

    impl FromRef<TestState> for Arc<BlogPosts> {
        fn from_ref(s: &TestState) -> Self {
            s.blog.clone()
        }
    }

    impl FromRef<TestState> for Arc<PodcastEpisodes> {
        fn from_ref(s: &TestState) -> Self {
            s.podcast.clone()
        }
    }

    fn test_app() -> (Router, Arc<BlogPosts>, Arc<PodcastEpisodes>) {
        let blog = Arc::new(BlogPosts::from_static_dir().unwrap());
        let podcast = Arc::new(PodcastEpisodes::from_static_dir().unwrap());
        let state = TestState {
            blog: blog.clone(),
            podcast: podcast.clone(),
        };
        let router = Router::new()
            .route("/og/posts/{slug}", get(og_post_svg))
            .route("/og/podcast/{slug}", get(og_podcast_svg))
            .route("/og/weekly/{slug}", get(og_weekly_svg))
            .with_state(state);
        (router, blog, podcast)
    }

    async fn body_string(resp: axum::response::Response) -> String {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8_lossy(&bytes).to_string()
    }

    #[tokio::test]
    async fn og_post_svg_returns_svg_for_known_regular_slug() {
        let (app, blog, _) = test_app();
        let regular = blog
            .posts()
            .iter()
            .find(|p| !p.frontmatter.is_newsletter)
            .expect("at least one non-newsletter post in fixtures");
        let slug = regular.og_slug();

        let resp = app
            .oneshot(
                Request::builder()
                    .uri(format!("/og/posts/{slug}.svg"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp
            .headers()
            .get("content-type")
            .map(|v| v.to_str().unwrap().to_string())
            .unwrap_or_default();
        assert!(
            ct.starts_with("image/svg+xml"),
            "unexpected content-type: {ct}"
        );
        let body = body_string(resp).await;
        assert!(body.contains("<svg"));
    }

    #[tokio::test]
    async fn og_post_svg_404s_for_unknown_slug() {
        let (app, _, _) = test_app();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/og/posts/this-slug-does-not-exist-anywhere.svg")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn og_post_and_weekly_pair_for_known_newsletter_slug() {
        let (app, blog, _) = test_app();
        let newsletter_slug = "20230713";
        let nl = blog
            .posts()
            .iter()
            .find(|p| p.frontmatter.is_newsletter && p.og_slug() == newsletter_slug);
        assert!(
            nl.is_some(),
            "fixture invariant: 20230713 is a newsletter post"
        );

        let resp_post = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/og/posts/{newsletter_slug}.svg"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp_post.status(), StatusCode::NOT_FOUND);

        let resp_weekly = app
            .oneshot(
                Request::builder()
                    .uri(format!("/og/weekly/{newsletter_slug}.svg"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp_weekly.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn og_weekly_svg_404s_for_regular_slug() {
        let (app, blog, _) = test_app();
        let regular = blog
            .posts()
            .iter()
            .find(|p| !p.frontmatter.is_newsletter)
            .expect("at least one non-newsletter post in fixtures");
        let slug = regular.og_slug();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri(format!("/og/weekly/{slug}.svg"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn og_podcast_svg_renders() {
        let (app, _, podcast) = test_app();
        let ep = podcast.episodes.first().expect("at least one episode");
        let slug = &ep.frontmatter.slug;
        let resp = app
            .oneshot(
                Request::builder()
                    .uri(format!("/og/podcast/{slug}.svg"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // 200 regardless of whether YT thumbnail fetch succeeded — CI may have no outbound net.
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
