use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use cja::app_state::AppState as _;
use db::tool_suggestions::ToolSuggestion;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    http_server::{auth::session::AdminUser, errors::WithStatus as _, ResponseResult},
    AppState,
};

#[derive(Serialize, Deserialize)]
pub struct ListToolSuggestionsResponse {
    suggestions: Vec<ToolSuggestion>,
}

#[derive(Deserialize)]
pub struct DismissRequest {
    linear_ticket_id: String,
}

#[axum_macros::debug_handler]
pub async fn list_pending_suggestions(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> ResponseResult<impl IntoResponse> {
    let suggestions = ToolSuggestion::list_pending(state.db())
        .await
        .context("Failed to fetch pending tool suggestions")
        .with_status(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ListToolSuggestionsResponse { suggestions }))
}

use color_eyre::eyre::Context;

#[axum_macros::debug_handler]
pub async fn dismiss_suggestion(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<DismissRequest>,
) -> ResponseResult<impl IntoResponse> {
    let suggestion = ToolSuggestion::get_by_id(state.db(), id)
        .await
        .context("Failed to fetch tool suggestion")
        .with_status(StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or_else(|| color_eyre::eyre::eyre!("Tool suggestion not found"))
        .with_status(StatusCode::NOT_FOUND)?;

    if suggestion.status != "pending" {
        return Err(color_eyre::eyre::eyre!("Tool suggestion is not pending"))
            .with_status(StatusCode::BAD_REQUEST);
    }

    let updated = ToolSuggestion::dismiss(state.db(), id, payload.linear_ticket_id)
        .await
        .context("Failed to dismiss tool suggestion")
        .with_status(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(updated))
}

#[axum_macros::debug_handler]
pub async fn skip_suggestion(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ResponseResult<impl IntoResponse> {
    let suggestion = ToolSuggestion::get_by_id(state.db(), id)
        .await
        .context("Failed to fetch tool suggestion")
        .with_status(StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or_else(|| color_eyre::eyre::eyre!("Tool suggestion not found"))
        .with_status(StatusCode::NOT_FOUND)?;

    if suggestion.status != "pending" {
        return Err(color_eyre::eyre::eyre!("Tool suggestion is not pending"))
            .with_status(StatusCode::BAD_REQUEST);
    }

    let updated = ToolSuggestion::skip(state.db(), id)
        .await
        .context("Failed to skip tool suggestion")
        .with_status(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(updated))
}
