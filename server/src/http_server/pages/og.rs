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
    //! These tests exercise the real `make_router()` via `test_helpers::create_test_app`
    //! so route wiring, the `{slug}` path matcher, and `.svg` suffix stripping are all
    //! verified end-to-end against the same router that ships in production.
    use crate::http_server::test_helpers::create_test_app;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use posts::{blog::BlogPosts, podcast::PodcastEpisodes};
    use tower::ServiceExt;

    fn fixtures() -> (BlogPosts, PodcastEpisodes) {
        (
            BlogPosts::from_static_dir().unwrap(),
            PodcastEpisodes::from_static_dir().unwrap(),
        )
    }

    async fn body_string(resp: axum::response::Response) -> String {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8_lossy(&bytes).to_string()
    }

    #[tokio::test]
    async fn og_post_svg_returns_svg_for_known_regular_slug() {
        let app = create_test_app().await;
        let (blog, _) = fixtures();
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
        let app = create_test_app().await;
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
        let app = create_test_app().await;
        let (blog, _) = fixtures();
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
        let app = create_test_app().await;
        let (blog, _) = fixtures();
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
        let app = create_test_app().await;
        let (_, podcast) = fixtures();
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
