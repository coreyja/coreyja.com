use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
};
use cja::jobs::Job;
use miette::IntoDiagnostic;

use crate::{
    http_server::{auth::session::AdminUser, errors::MietteError},
    jobs::youtube_videos::RefreshVideos,
    state::AppState,
};

pub(crate) async fn refresh_youtube(
    State(app_state): State<AppState>,
    _admin: AdminUser,
) -> Result<impl IntoResponse, MietteError> {
    RefreshVideos
        .enqueue(app_state, "Admin Dashboard Manual".to_string())
        .await
        .into_diagnostic()?;

    Ok(Redirect::to("/admin"))
}
