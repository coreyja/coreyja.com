use axum::{extract::State, response::IntoResponse};
use include_dir::{include_dir, Dir};

use crate::state::AppState;

use super::super::{auth::session::AdminUser, errors::ServerError};

const THREAD_FRONTEND_DIST: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../thread-frontend/dist");

pub(crate) async fn threads_app(
    _admin: AdminUser,
    State(_app_state): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
    // Serve the index.html file
    let index_html = THREAD_FRONTEND_DIST
        .get_file("index.html")
        .ok_or_else(|| color_eyre::eyre::eyre!("index.html not found"))?;

    let content = index_html
        .contents_utf8()
        .ok_or_else(|| color_eyre::eyre::eyre!("Failed to read index.html"))?;

    Ok(axum::response::Html(content))
}

pub(crate) async fn serve_thread_assets(
    _admin: AdminUser,
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    // Try to find the file in the embedded directory
    let file = THREAD_FRONTEND_DIST
        .get_file(&path)
        .ok_or_else(|| color_eyre::eyre::eyre!("File not found"))?;

    // Determine content type based on file extension
    let content_type = match path.split('.').next_back() {
        Some("js") => "application/javascript",
        Some("css") => "text/css",
        Some("html") => "text/html",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    };

    let contents = file.contents();

    Ok((
        axum::http::StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, content_type)],
        contents,
    ))
}
