use axum::response::IntoResponse;
use miette::Diagnostic;
use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Diagnostic, Error)]
#[error("MietteError")]
pub struct MietteError(pub(crate) miette::Report);

impl IntoResponse for MietteError {
    fn into_response(self) -> axum::response::Response {
        sentry::capture_error(&self);

        (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}

impl From<miette::Report> for MietteError {
    fn from(err: miette::Report) -> Self {
        MietteError(err)
    }
}
