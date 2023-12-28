use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use maud::{html, Markup, Render};
use miette::IntoDiagnostic;
use uuid::Uuid;

use crate::{
    http_server::templates::{base_constrained, header::OpenGraph},
    state::AppState,
};

pub(crate) struct YoutubeVideo {
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    pub(crate) youtube_video_id: String,
    pub(crate) external_youtube_id: String,
    pub(crate) thumbnail_url: Option<String>,
    pub(crate) published_at: Option<chrono::NaiveDateTime>,
}

pub(crate) struct VideoList(pub(crate) Vec<YoutubeVideo>);

impl Render for VideoList {
    fn render(&self) -> Markup {
        html! {
          ul {
            @for video in &self.0 {
              li class="my-8" {

                a href=(format!("/videos/{}", video.youtube_video_id)) {
                    img class="h-[180px] aspect-video object-cover object-center mb-2" src=(video.thumbnail_url.as_deref().unwrap()) alt=(video.title);
                    p class="" { (video.title) }

                    p class="text-subtitle text-sm" { (video.published_at.unwrap().date()) }
                }
              }
            }
          }
        }
    }
}

pub(crate) async fn video_get(
    Path(id): Path<Uuid>,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, crate::http_server::errors::MietteError> {
    let video = sqlx::query_as!(
        YoutubeVideo,
        "SELECT *
    FROM YoutubeVideos
    WHERE youtube_video_id = $1",
        id
    )
    .fetch_optional(&app_state.db)
    .await
    .into_diagnostic()?;

    Ok(base_constrained(
        html! {
          @if let Some(video) = video {
            (video.title)
            h1 class="text-2xl" { (video.title) }
            @if let Some(published_at) = video.published_at {
              subtitle class="block text-lg text-subtitle mb-8 " { (published_at.format("%Y-%m-%d")) }
            }

            // @if let Some(project) = project {
            //   div class="mb-8" {
            //     "Project: "
            //     a href=(project.relative_link().map_err(|e| MietteError(e, StatusCode::INTERNAL_SERVER_ERROR)).map_err(|e| e.into_response())?) { (project.frontmatter.title) }
            //   }
            // }

            iframe
              id="ytplayer"
              type="text/html"
              width="640"
              height="360"
              src=(format!("https://www.youtube.com/embed/{}", video.external_youtube_id))
              frameborder="0"
              {}

            @if let Some(description) = video.description {
              div class="whitespace-pre-wrap max-w-prose my-4" {
                (description)
              }
            }
          }
        },
        OpenGraph::default(),
    ))
}
