use axum::http::StatusCode;
use axum::response::IntoResponse;
use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Diagnostic, Error)]
#[error("MietteError: ${0}")]
pub struct MietteError(pub(crate) miette::Report, pub(crate) StatusCode);

impl IntoResponse for MietteError {
    fn into_response(self) -> axum::response::Response {
        sentry::capture_error(&self);
        tracing::error!(error = %self.0, "MietteError");

        (self.1, self.0.to_string()).into_response()
    }
}

impl From<miette::Report> for MietteError {
    fn from(err: miette::Report) -> Self {
        MietteError(err, StatusCode::INTERNAL_SERVER_ERROR)
    }
}
