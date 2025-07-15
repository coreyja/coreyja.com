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

#[derive(Serialize, Deserialize)]
struct ThreadWithStitches {
    #[serde(flatten)]
    thread: Thread,
    stitches: Vec<Stitch>,
}

#[derive(Serialize, Deserialize)]
struct ThreadsListResponse {
    threads: Vec<Thread>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http_server::test_helpers::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use db::test_utils::create_test_db;
    use tower::util::ServiceExt;
    
    async fn setup_test_thread(pool: &sqlx::PgPool) -> Thread {
        Thread::create(pool, "Test thread goal".to_string())
            .await
            .expect("Failed to create test thread")
    }

    #[tokio::test]
    async fn test_list_threads_requires_admin() {
        let pool = create_test_db().await;
        let app = create_test_app(pool).await;

        let request = Request::builder()
            .uri("/admin/api/threads")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_list_threads_empty() {
        let pool = create_test_db().await;
        let app = create_test_app(pool).await;

        let request = admin_request_builder()
            .uri("/admin/api/threads")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = response_body_json::<ThreadsListResponse>(response).await;
        assert_eq!(body.threads.len(), 0);
    }

    #[tokio::test]
    async fn test_list_threads_with_data() {
        let pool = create_test_db().await;
        let thread1 = setup_test_thread(&pool).await;
        let thread2 = setup_test_thread(&pool).await;
        
        let app = create_test_app(pool).await;

        let request = admin_request_builder()
            .uri("/admin/api/threads")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = response_body_json::<ThreadsListResponse>(response).await;
        assert_eq!(body.threads.len(), 2);
        
        // Should be ordered by created_at DESC
        assert_eq!(body.threads[0].thread_id, thread2.thread_id);
        assert_eq!(body.threads[1].thread_id, thread1.thread_id);
    }

    #[tokio::test]
    async fn test_get_thread_not_found() {
        let pool = create_test_db().await;
        let app = create_test_app(pool).await;

        let request = admin_request_builder()
            .uri("/admin/api/threads/00000000-0000-0000-0000-000000000000")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_thread_with_stitches() {
        let pool = create_test_db().await;
        let thread = setup_test_thread(&pool).await;
        
        // Add some stitches
        let stitch1 = Stitch::create_tool_call(
            &pool,
            thread.thread_id,
            None,
            "test_tool".to_string(),
            serde_json::json!({"input": "test"}),
            serde_json::json!({"output": "result"})
        ).await.unwrap();

        // For testing, we'll use create_tool_call for the second stitch
        let stitch2 = Stitch::create_tool_call(
            &pool,
            thread.thread_id,
            Some(stitch1.stitch_id),
            "test_tool_2".to_string(),
            serde_json::json!({"input": "test2"}),
            serde_json::json!({"output": "result2"})
        ).await.unwrap();
        
        let app = create_test_app(pool).await;

        let request = admin_request_builder()
            .uri(&format!("/admin/api/threads/{}", thread.thread_id))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = response_body_json::<ThreadWithStitches>(response).await;
        assert_eq!(body.thread.thread_id, thread.thread_id);
        assert_eq!(body.stitches.len(), 2);
        assert_eq!(body.stitches[0].stitch_id, stitch1.stitch_id);
        assert_eq!(body.stitches[1].stitch_id, stitch2.stitch_id);
    }

    #[tokio::test]
    async fn test_create_thread_success() {
        let pool = create_test_db().await;
        let app = create_test_app(pool.clone()).await;

        let payload = CreateThreadRequest {
            goal: "New test thread".to_string(),
        };

        let request = admin_request_builder()
            .method("POST")
            .uri("/admin/api/threads")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let body = response_body_json::<Thread>(response).await;
        assert_eq!(body.goal, "New test thread");
        assert_eq!(body.status, "pending");
        
        // Verify it was saved to database
        let saved = Thread::get_by_id(&pool, body.thread_id).await.unwrap().unwrap();
        assert_eq!(saved.goal, "New test thread");
    }

    #[tokio::test]
    async fn test_create_thread_empty_goal() {
        let pool = create_test_db().await;
        let app = create_test_app(pool).await;

        let payload = CreateThreadRequest {
            goal: "".to_string(),
        };

        let request = admin_request_builder()
            .method("POST")
            .uri("/admin/api/threads")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        // Should still create - validation can be added later if needed
        assert_eq!(response.status(), StatusCode::CREATED);
    }
}
