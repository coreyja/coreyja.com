use std::fmt::{Debug, Display};

use axum::http::StatusCode;
use axum::response::IntoResponse;

#[derive(Debug)]
pub struct ServerError(pub(crate) cja::color_eyre::Report, pub(crate) StatusCode);

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!(error = ?self, "Error");

        (self.1, self.0.to_string()).into_response()
    }
}

impl<E> From<E> for ServerError
where
    E: Into<cja::color_eyre::Report>,
{
    fn from(err: E) -> Self {
        ServerError(err.into(), StatusCode::INTERNAL_SERVER_ERROR)
    }
}
