use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
};
use cja::jobs::Job;

use crate::{
    http_server::{auth::session::AdminUser, errors::ServerError},
    jobs::youtube_videos::RefreshVideos,
    state::AppState,
};

pub(crate) async fn refresh_youtube(
    State(app_state): State<AppState>,
    _admin: AdminUser,
) -> cja::Result<impl IntoResponse, ServerError> {
    RefreshVideos
        .enqueue(app_state, "Admin Dashboard Manual".to_string())
        .await?;

    Ok(Redirect::to("/admin"))
}
