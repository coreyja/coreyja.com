use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use cja::app_state::AppState as _;
use db::agentic_threads::{Stitch, Thread};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    http_server::{auth::session::AdminUser, errors::WithStatus as _, ResponseResult},
    AppState,
};

#[derive(Serialize)]
struct ThreadWithStitches {
    #[serde(flatten)]
    thread: Thread,
    stitches: Vec<Stitch>,
}

#[derive(Serialize)]
struct ThreadsListResponse {
    threads: Vec<Thread>,
}

#[derive(Deserialize)]
pub(crate) struct CreateThreadRequest {
    goal: String,
}

#[axum_macros::debug_handler]
pub async fn list_threads(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> ResponseResult<impl IntoResponse> {
    let threads = Thread::list_all(state.db())
        .await
        .context("Failed to fetch threads")
        .with_status(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ThreadsListResponse { threads }))
}

#[axum_macros::debug_handler]
pub async fn get_thread(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ResponseResult<impl IntoResponse> {
    let thread = Thread::get_by_id(state.db(), id)
        .await
        .context("Failed to fetch thread")
        .with_status(StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or_else(|| color_eyre::eyre::eyre!("Thread not found"))?;

    let stitches = thread
        .get_stitches(state.db())
        .await
        .context("Failed to fetch stitches")
        .with_status(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ThreadWithStitches { thread, stitches }))
}

#[axum_macros::debug_handler]
pub async fn create_thread(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateThreadRequest>,
) -> ResponseResult<impl IntoResponse> {
    let thread = Thread::create(state.db(), payload.goal)
        .await
        .context("Failed to create thread")
        .with_status(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(thread)))
}

use color_eyre::eyre::Context;
