use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
};

use crate::{
    http_server::{auth::session::AdminUser, errors::MietteError},
    jobs::{youtube_videos::RefreshVideos, Job},
    state::AppState,
};

pub(crate) async fn refresh_youtube(
    State(app_state): State<AppState>,
    _admin: AdminUser,
) -> Result<impl IntoResponse, MietteError> {
    RefreshVideos
        .enqueue(app_state, "Admin Dashboard Manual".to_string())
        .await?;

    Ok(Redirect::to("/admin"))
}
