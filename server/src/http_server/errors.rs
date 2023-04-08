use axum::response::IntoResponse;

pub struct EyreError(color_eyre::Report);

impl IntoResponse for EyreError {
    fn into_response(self) -> axum::response::Response {
        self.0.to_string().into_response()
    }
}

impl<T> From<T> for EyreError
where
    T: Into<color_eyre::Report>,
{
    fn from(err: T) -> Self {
        EyreError(err.into())
    }
}
