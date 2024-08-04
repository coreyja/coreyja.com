use axum::{extract::State, response::IntoResponse};

use crate::{
    http_server::{
        errors::ServerError,
        templates::{base_constrained, header::OpenGraph},
    },
    AppState,
};

pub(crate) async fn iframe_demo(
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
    Ok(base_constrained(
        maud::html! {
          h1 { "Cookd Demo" }

          iframe class="w-full min-h-screen" src="https://coreyja.cookd.dev/level-0-0" {}
        },
        OpenGraph::default(),
    ))
}
