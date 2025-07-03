use axum::{extract::State, response::IntoResponse};
use cja::server::session::AppSession;
use cja::server::session::Session;
use maud::html;

use crate::{http_server::auth::session::DBSession, AppState};

#[allow(clippy::unused_async)]
pub(crate) async fn versions(
    State(app): State<AppState>,
    Session(session): Session<DBSession>,
) -> impl IntoResponse {
    html! {
      p { "coreyja.com " }
      p { "Session Id: " (session.session_id()) }
      @if let Some(user_id) = session.user_id {
        p { "User Id: " (user_id) }
      }
      p { "Git Commit: " (app.versions.git_commit) }
      p { "Rust Version: " (app.versions.rustc_version) }
    }
}
