use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

use crate::*;

pub(crate) async fn upwork_proposal_get(
    Path(id): Path<String>,
    State(config): State<Config>,
) -> Result<impl IntoResponse, http_server::EyreError> {
    Ok(id)
}
