use std::fmt::{Debug, Display, Write};

use axum::http::StatusCode;
use axum::response::IntoResponse;
use miette::{Diagnostic, EyreContext, NarratableReportHandler};
use thiserror::Error;

#[derive(Diagnostic, Error)]
pub struct MietteError(pub(crate) miette::Report, pub(crate) StatusCode);

impl Display for MietteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let handler = NarratableReportHandler::new();

        handler.display(self.0.as_ref(), f)
    }
}

impl Debug for MietteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let handler = NarratableReportHandler::new();

        f.write_fmt(format_args!("Status Code: {}", self.1))?;
        f.write_str("MietteError: \n")?;

        handler.debug(self.0.as_ref(), f)
    }
}

impl IntoResponse for MietteError {
    fn into_response(self) -> axum::response::Response {
        sentry::capture_error(&self);

        tracing::error!(error = ?self, "MietteError");

        (self.1, self.0.to_string()).into_response()
    }
}

impl From<miette::Report> for MietteError {
    fn from(err: miette::Report) -> Self {
        MietteError(err, StatusCode::INTERNAL_SERVER_ERROR)
    }
}
