use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
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
) -> Result<Response, ServerError> {
    // Try to find the file in the embedded directory
    let Some(file) = THREAD_FRONTEND_DIST.get_file(&path) else {
        // Serve the index.html file
        let index_html = THREAD_FRONTEND_DIST
            .get_file("index.html")
            .ok_or_else(|| color_eyre::eyre::eyre!("index.html not found"))?;

        let content = index_html
            .contents_utf8()
            .ok_or_else(|| color_eyre::eyre::eyre!("Failed to read index.html"))?;

        return Ok(axum::response::Html(content).into_response());
    };

    // Determine content type based on file extension
    let mime = mime_guess::from_path(&path).first_or_octet_stream();

    let contents = file.contents();

    Ok((
        axum::http::StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, mime.to_string())],
        contents,
    )
        .into_response())
}
