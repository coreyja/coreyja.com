use axum::{extract::State, response::IntoResponse};
use cja::server::session::DBSession;
use maud::html;

use crate::AppState;

#[axum_macros::debug_handler]
pub(crate) async fn versions(
    State(app): State<AppState>,
    session: Option<DBSession>,
) -> impl IntoResponse {
    html! {
      p { "coreyja.com " }
      @if let Some(session) = session {
        p { "Session Id: " (session.session_id) }
        p { "User Id: " (session.user_id) }
      }
      p { "Git Commit: " (app.versions.git_commit) }
      p { "Rust Version: " (app.versions.rustc_version) }
    }
}
