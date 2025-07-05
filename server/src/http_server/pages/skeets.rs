use axum::{
    extract::State,
    response::IntoResponse,
};
use db::skeets::Skeet;
use maud::{html, Markup, Render};

use crate::{
    http_server::templates::{base_constrained, header::OpenGraph},
    state::AppState,
};

pub(crate) struct SkeetList(pub(crate) Vec<Skeet>);

impl Render for SkeetList {
    fn render(&self) -> Markup {
        html! {
            ul class="space-y-6" {
                @for skeet in &self.0 {
                    (SkeetCard(skeet))
                }
            }
        }
    }
}

pub(crate) struct SkeetCard<'a>(pub(crate) &'a Skeet);

impl Render for SkeetCard<'_> {
    fn render(&self) -> Markup {
        let skeet = self.0;
        
        html! {
            li class="border border-neutral-700 rounded-lg p-4" {
                p class="whitespace-pre-wrap" { (skeet.content) }
                
                @if let Some(published_at) = skeet.published_at {
                    p class="text-subtitle text-sm mt-2" { 
                        "Posted: " (published_at.format("%Y-%m-%d %H:%M")) 
                    }
                }
            }
        }
    }
}

pub(crate) async fn skeets_index(
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, crate::http_server::errors::ServerError> {
    let skeets = sqlx::query_as!(
        Skeet,
        r#"SELECT *
        FROM Skeets
        WHERE published_at IS NOT NULL
        ORDER BY published_at DESC"#
    )
    .fetch_all(&app_state.db)
    .await?;

    Ok(base_constrained(
        html! {
            h1 class="text-3xl mb-6" { "Skeets" }
            p class="mb-8" { 
                "Short updates and thoughts that also get published to Bluesky, Twitter, and other social platforms."
            }

            (SkeetList(skeets))
        },
        OpenGraph::default(),
    ))
}