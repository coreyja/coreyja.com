use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use cja::app_state::AppState as _;
use db::agentic_threads::{Stitch, Thread, ThreadType};
use db::discord_threads::DiscordThreadMetadata;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    http_server::{auth::session::AdminUser, errors::WithStatus as _, ResponseResult},
    AppState,
};

#[derive(Serialize, Deserialize)]
struct ThreadWithStitches {
    #[serde(flatten)]
    thread: Thread,
    stitches: Vec<Stitch>,
    discord_metadata: Option<DiscordThreadMetadata>,
}

#[derive(Serialize, Deserialize)]
struct ThreadsListResponse {
    threads: Vec<Thread>,
}

#[derive(Serialize, Deserialize)]
struct ThreadWithCounts {
    #[serde(flatten)]
    thread: Thread,
    stitch_count: i64,
    children_count: i64,
}

#[derive(Serialize, Deserialize)]
struct ThreadsWithCountsResponse {
    threads: Vec<ThreadWithCounts>,
}

#[derive(Serialize, Deserialize)]
struct ChildrenResponse {
    children: Vec<ThreadWithCounts>,
}

#[derive(Deserialize, Serialize)]
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

    let mut threads_with_counts = Vec::new();
    for thread in threads {
        let stitch_count = thread
            .count_stitches(state.db())
            .await
            .context("Failed to count stitches")
            .with_status(StatusCode::INTERNAL_SERVER_ERROR)?;

        let children_count = thread
            .count_children(state.db())
            .await
            .context("Failed to count children")
            .with_status(StatusCode::INTERNAL_SERVER_ERROR)?;

        threads_with_counts.push(ThreadWithCounts {
            thread,
            stitch_count,
            children_count,
        });
    }

    Ok(Json(ThreadsWithCountsResponse {
        threads: threads_with_counts,
    }))
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

    // Fetch Discord metadata if this is an interactive thread
    let discord_metadata = if thread.thread_type == ThreadType::Interactive {
        DiscordThreadMetadata::find_by_thread_id(state.db(), thread.thread_id)
            .await
            .context("Failed to fetch Discord metadata")
            .with_status(StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        None
    };

    Ok(Json(ThreadWithStitches {
        thread,
        stitches,
        discord_metadata,
    }))
}

#[axum_macros::debug_handler]
pub async fn get_thread_messages(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ResponseResult<impl IntoResponse> {
    let messages = crate::jobs::thread_processor::reconstruct_messages(state.db(), id).await?;

    Ok(Json(messages))
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

#[axum_macros::debug_handler]
pub async fn get_thread_parents(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ResponseResult<impl IntoResponse> {
    let thread = Thread::get_by_id(state.db(), id)
        .await
        .context("Failed to fetch thread")
        .with_status(StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or_else(|| color_eyre::eyre::eyre!("Thread not found"))?;

    let parents = thread
        .get_parent_chain(state.db())
        .await
        .context("Failed to fetch parent chain")
        .with_status(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(parents))
}

#[axum_macros::debug_handler]
pub async fn list_recent_threads(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> ResponseResult<impl IntoResponse> {
    let threads = Thread::list_recent_top_level(state.db(), 20)
        .await
        .context("Failed to fetch recent threads")
        .with_status(StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut threads_with_counts = Vec::new();
    for thread in threads {
        let stitch_count = thread
            .count_stitches(state.db())
            .await
            .context("Failed to count stitches")
            .with_status(StatusCode::INTERNAL_SERVER_ERROR)?;

        let children_count = thread
            .count_children(state.db())
            .await
            .context("Failed to count children")
            .with_status(StatusCode::INTERNAL_SERVER_ERROR)?;

        threads_with_counts.push(ThreadWithCounts {
            thread,
            stitch_count,
            children_count,
        });
    }

    Ok(Json(ThreadsWithCountsResponse {
        threads: threads_with_counts,
    }))
}

#[axum_macros::debug_handler]
pub async fn get_thread_children(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ResponseResult<impl IntoResponse> {
    let thread = Thread::get_by_id(state.db(), id)
        .await
        .context("Failed to fetch thread")
        .with_status(StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or_else(|| color_eyre::eyre::eyre!("Thread not found"))?;

    let children = thread
        .get_children(state.db())
        .await
        .context("Failed to fetch children")
        .with_status(StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut children_with_counts = Vec::new();
    for child in children {
        let stitch_count = child
            .count_stitches(state.db())
            .await
            .context("Failed to count stitches")
            .with_status(StatusCode::INTERNAL_SERVER_ERROR)?;

        let children_count = child
            .count_children(state.db())
            .await
            .context("Failed to count children")
            .with_status(StatusCode::INTERNAL_SERVER_ERROR)?;

        children_with_counts.push(ThreadWithCounts {
            thread: child,
            stitch_count,
            children_count,
        });
    }

    Ok(Json(ChildrenResponse {
        children: children_with_counts,
    }))
}
