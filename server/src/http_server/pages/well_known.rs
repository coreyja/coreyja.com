//! `.well-known` endpoints for standard.site discovery / verification.
//!
//! Per <https://standard.site/docs/verification/>, a publication served at
//! `https://example.com/path/to/pub` must answer
//! `https://example.com/.well-known/site.standard.publication/path/to/pub`
//! with the publication's AT URI as plain text. For a publication that
//! lives at the domain root, the endpoint is the bare
//! `/.well-known/site.standard.publication`.

use std::sync::LazyLock;

use axum::{
    extract::Path,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};

static PUBLICATIONS_TOML: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../publications.toml"));

#[derive(serde::Deserialize)]
struct PublicationsFile {
    publication: Vec<PublicationWellKnownStub>,
}

#[derive(serde::Deserialize)]
struct PublicationWellKnownStub {
    url: String,
    at_uri: Option<String>,
}

static PUBLICATION_STUBS: LazyLock<Vec<PublicationWellKnownStub>> = LazyLock::new(|| {
    toml::from_str::<PublicationsFile>(PUBLICATIONS_TOML)
        .expect("publications.toml must parse")
        .publication
});

/// Return the URL path component of a publication's `url` (e.g.
/// `https://coreyja.com/posts` → `/posts`). Trailing slashes are stripped
/// so `/posts` and `/posts/` match identically.
fn publication_path(url: &str) -> String {
    let after_scheme = url.split_once("://").map_or(url, |(_, rest)| rest);
    let path = match after_scheme.find('/') {
        Some(i) => &after_scheme[i..],
        None => "",
    };
    path.trim_end_matches('/').to_string()
}

fn plain_text_response(body: String) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "text/plain; charset=utf-8".parse().unwrap(),
    );
    (headers, body).into_response()
}

fn lookup_at_uri_for_path(request_path: &str) -> Option<String> {
    let normalized = request_path.trim_end_matches('/');
    PUBLICATION_STUBS
        .iter()
        .find(|p| publication_path(&p.url) == normalized)
        .and_then(|p| p.at_uri.clone())
}

/// Handles `/.well-known/site.standard.publication` (no path suffix).
/// Matches the publication whose URL lives at the domain root.
pub async fn well_known_publication_root() -> Result<Response, StatusCode> {
    let at_uri = lookup_at_uri_for_path("").ok_or(StatusCode::NOT_FOUND)?;
    Ok(plain_text_response(at_uri))
}

/// Handles `/.well-known/site.standard.publication/{*path}`. The wildcard
/// captures everything after the prefix; we re-add the leading slash so it
/// matches the `publication_path` form (`/posts`, `/blog`, ...).
pub async fn well_known_publication_with_path(
    Path(path): Path<String>,
) -> Result<Response, StatusCode> {
    let request_path = format!("/{}", path.trim_start_matches('/'));
    let at_uri = lookup_at_uri_for_path(&request_path).ok_or(StatusCode::NOT_FOUND)?;
    Ok(plain_text_response(at_uri))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publication_path_strips_scheme_and_trailing_slash() {
        assert_eq!(publication_path("https://coreyja.com/posts"), "/posts");
        assert_eq!(publication_path("https://coreyja.com/posts/"), "/posts");
        assert_eq!(publication_path("https://coreyja.com/"), "");
        assert_eq!(publication_path("https://coreyja.com"), "");
        assert_eq!(
            publication_path("https://example.com/path/to/pub"),
            "/path/to/pub"
        );
    }
}
