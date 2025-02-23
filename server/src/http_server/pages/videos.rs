use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use maud::{html, Markup, Render};
use uuid::Uuid;

use crate::{
    http_server::templates::{base_constrained, header::OpenGraph},
    state::AppState,
};

pub(crate) struct YoutubeVideo {
    pub(crate) description: Option<String>,
    pub(crate) external_youtube_id: String,
    pub(crate) published_at: Option<chrono::NaiveDateTime>,
    pub(crate) thumbnail_url: Option<String>,
    pub(crate) title: String,
    #[allow(clippy::struct_field_names)]
    pub(crate) youtube_video_id: String,
}

pub(crate) struct VideoList(pub(crate) Vec<YoutubeVideo>);

impl Render for VideoList {
    fn render(&self) -> Markup {
        html! {
          ul class="flex flex-row flex-wrap" {
            @for video in &self.0 {
              (VideoThumbnailCard(video))
            }
          }
        }
    }
}

pub(crate) struct VideoThumbnailCard<'a>(pub(crate) &'a YoutubeVideo);

impl Render for VideoThumbnailCard<'_> {
    fn render(&self) -> Markup {
        let video = &self.0;

        html! {
          li class="my-8" {

            a href=(format!("/videos/{}", video.youtube_video_id)) {
                img class="h-[180px] aspect-video object-cover object-center mb-2" src=(video.thumbnail_url.as_deref().unwrap()) alt=(video.title) loading="lazy";
                p class="max-w-[340px]" { (video.title) }

                p class="text-subtitle text-sm" { (video.published_at.unwrap().date()) }
            }
          }
        }
    }
}

pub(crate) async fn video_index(
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, crate::http_server::errors::ServerError> {
    let videos = sqlx::query_as!(
        YoutubeVideo,
        "SELECT *
      FROM YoutubeVideos
      ORDER BY published_at DESC"
    )
    .fetch_all(&app_state.db)
    .await?;

    Ok(base_constrained(
        html! {
          h1 class="text-3xl" { "Past Videos" }

          ul class="grid grid-cols-1 md:grid-cols-2 gap-4" {
            @for video in &videos {
              (VideoThumbnailCard(video))
            }
          }
        },
        OpenGraph::default(),
    ))
}

pub(crate) async fn video_get(
    Path(id): Path<Uuid>,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, crate::http_server::errors::ServerError> {
    let video = sqlx::query_as!(
        YoutubeVideo,
        "SELECT *
    FROM YoutubeVideos
    WHERE youtube_video_id = $1",
        id
    )
    .fetch_optional(&app_state.db)
    .await?;

    Ok(base_constrained(
        html! {
          @if let Some(video) = video {
            h1 class="text-2xl" { (video.title) }
            @if let Some(published_at) = video.published_at {
              subtitle class="block text-lg text-subtitle mb-8 " { (published_at.format("%Y-%m-%d")) }
            }

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
