use axum::response::IntoResponse;

pub struct MietteError(miette::Report);

impl IntoResponse for MietteError {
    fn into_response(self) -> axum::response::Response {
        self.0.to_string().into_response()
    }
}

impl<T> From<T> for MietteError
where
    T: Into<miette::Report>,
{
    fn from(err: T) -> Self {
        MietteError(err.into())
    }
}
