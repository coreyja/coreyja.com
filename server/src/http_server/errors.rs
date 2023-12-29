use axum::http::StatusCode;
use axum::response::IntoResponse;
use maud::html;
use miette::{DebugReportHandler, Diagnostic, ErrReport, EyreContext};
use thiserror::Error;

use super::templates::base_constrained;

#[derive(Debug, Diagnostic, Error)]
#[error("MietteError: {0}")]
pub struct MietteError(
    #[diagnostic_source] pub(crate) miette::Report,
    pub(crate) StatusCode,
);

impl IntoResponse for MietteError {
    fn into_response(self) -> axum::response::Response {
        sentry::capture_error(&self);

        (
            self.1,
            base_constrained(
                html! {
                    h1 class="text-xl" { "Internal Server Error" }
                    p class="text-md text-gray-400 mb-2" { "Something went wrong." }


                    @for cause in self.0.chain() {
                        p { (cause) }
                    }
                },
                Default::default(),
            ),
        )
            .into_response()
    }
}

impl From<miette::Report> for MietteError {
    fn from(err: miette::Report) -> Self {
        MietteError(err, StatusCode::INTERNAL_SERVER_ERROR)
    }
}
