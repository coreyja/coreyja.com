use axum::{extract::State, response::IntoResponse};
use maud::html;

use crate::AppState;

#[axum_macros::debug_handler]
pub(crate) async fn versions(State(app): State<AppState>) -> impl IntoResponse {
    html! {
      p { "coreyja.com " }
      p { "Git Commit: " (app.versions.git_commit.unwrap_or("unknown")) }
      p { "Rust Version: " (rustc_version_runtime::version()) }
    }
}
