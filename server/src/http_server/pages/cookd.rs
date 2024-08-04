use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use cja::color_eyre::eyre::eyre;

use crate::{
    http_server::{
        errors::ServerError,
        templates::{base_constrained, header::OpenGraph},
        LinkTo,
    },
    AppState,
};

struct CookdLevel {
    slug: String,
    url: String,
}

fn get_cookd_levels() -> Vec<CookdLevel> {
    vec![
        CookdLevel {
            slug: "Level-0-0".to_string(),
            url: "https://coreyja.cookd.dev/level-0-0".to_string(),
        },
        CookdLevel {
            slug: "Level-1-1".to_string(),
            url: "https://corey.cookd.dev/level1-1".to_string(),
        },
    ]
}

impl LinkTo for CookdLevel {
    fn relative_link(&self) -> String {
        format!("/cookd_demo/{}", self.slug)
    }
}

pub(crate) async fn cookd_index(
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
    Ok(base_constrained(
        maud::html! {
          h1 { "Cookd Demo Index" }

          p { "This is a demo of the Cookd platform." }

          ul {
            @for cookd in get_cookd_levels() {
              li {
                a href=(cookd.relative_link()) { (cookd.slug) }
              }
            }
          }
        },
        OpenGraph::default(),
    ))
}

pub(crate) async fn cookd_get(
    Path(slug): Path<String>,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
    let cookd = get_cookd_levels();
    let cookd = cookd.into_iter().find(|c| c.slug == slug);
    let cookd =
        cookd.ok_or_else(|| ServerError(eyre!("Cookd level not found"), StatusCode::NOT_FOUND))?;

    Ok(base_constrained(
        maud::html! {
          h1 { "Cookd - "  (cookd.slug) }

          iframe class="w-full min-h-screen" src=(cookd.url) {}
        },
        OpenGraph::default(),
    ))
}
