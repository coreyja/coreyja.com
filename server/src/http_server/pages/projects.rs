use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use itertools::Itertools;
use maud::{html, Markup, Render};
use posts::projects::{Project, ProjectStatus, Projects};

use crate::{
    http_server,
    http_server::{
        errors::ServerError,
        templates::{base_constrained, header::OpenGraph},
        ResponseResult,
        pages::blog::md::html::MarkdownRenderContext,
    },
    instrument, AppState, Arc, Result,
};

use super::blog::md::IntoHtml;

#[instrument(skip_all)]
pub(crate) async fn projects_index(
    State(projects): State<Arc<Projects>>,
    State(state): State<AppState>,
) -> ResponseResult<Markup> {
    let projects = projects.by_title();

    let mut grouped_projects: Vec<(ProjectStatus, Vec<Project>)> = projects
        .into_iter()
        .map(|p| (p.frontmatter.status, p))
        .into_group_map()
        .into_iter()
        .collect::<Vec<_>>();

    grouped_projects.sort_by_key(|(status, _)| *status);

    let recent_video_published = sqlx::query!(
        r#"
        SELECT YoutubePlaylists.external_youtube_playlist_id,
        max(YoutubeVideos.published_at)
        FROM YoutubeVideos
        JOIN YoutubeVideoPlaylists using (youtube_video_id)
        JOIN YoutubePlaylists using (youtube_playlist_id)
        WHERE YoutubeVideos.published_at IS NOT NULL
        GROUP BY YoutubePlaylists.external_youtube_playlist_id
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| ServerError(e.into(), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(base_constrained(
        html! {
          h1 class="text-3xl mb-8" { "Projects" }

          @for (status, projects) in grouped_projects {
            (StatusTag(status))
            ul class="mb-8" {
              @for project in &projects {
                li class="my-4" {
                  a href=(project.relative_link()?) {
                    (project.frontmatter.title)

                    @let most_recent_video = recent_video_published
                                              .iter()
                                              .find(|s|
                                                Some(
                                                  &s.external_youtube_playlist_id) ==
                                                    project.frontmatter.youtube_playlist.as_ref()
                                              );
                    @if let Some(stream) = most_recent_video {
                      @if let Some(date) = stream.max {
                        span class="text-subtitle text-sm inline-block pl-4" {
                          "Most Recent Video: " (date.format("%Y-%m-%d"))
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        },
        OpenGraph::default(),
    ))
}

struct StatusTag(ProjectStatus);

impl StatusTag {
    fn color_class(&self) -> &'static str {
        match self.0 {
            ProjectStatus::Active => "fill-success-400",
            ProjectStatus::Maintenance => "fill-warning-400",
            ProjectStatus::OnIce => "fill-blue-400",
            ProjectStatus::Complete => "fill-success-200",
            ProjectStatus::Archived => "fill-grey-400",
        }
    }
}

impl Render for StatusTag {
    fn render(&self) -> Markup {
        html! {
          span class="inline-flex items-center gap-x-1.5 rounded-md px-2 py-1 text-xs font-medium text-text ring-1 ring-inset ring-grey-800" {
            svg class=(format!("h-1.5 w-1.5 {}", self.color_class())) viewBox="0 0 6 6" aria-hidden="true" {
              circle cx="3" cy="3" r="3";
            }
            (self.0)
          }
        }
    }
}

#[instrument(skip_all, fields(slug))]
#[axum_macros::debug_handler(state = AppState)]
pub(crate) async fn projects_get(
    State(projects): State<Arc<Projects>>,
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Markup, Response> {
    let project = projects
        .projects
        .iter()
        .find(|p| p.slug().unwrap() == slug)
        .ok_or_else(|| StatusCode::NOT_FOUND.into_response())?;

    let markdown = project
        .ast
        .0
        .clone()
        .into_html(&state.app, &MarkdownRenderContext { syntax_highlighting: state.syntax_highlighting_context.clone(), current_article_path: None })
        .map_err(|e| MietteError(e, StatusCode::INTERNAL_SERVER_ERROR))
        .map_err(axum::response::IntoResponse::into_response)?;

    let youtube_videos = sqlx::query_as!(
        crate::http_server::pages::videos::YoutubeVideo,
        r#"
        SELECT YoutubeVideos.*
        FROM YoutubeVideos
        JOIN YoutubeVideoPlaylists using (youtube_video_id)
        JOIN YoutubePlaylists using (youtube_playlist_id)
        WHERE YoutubePlaylists.external_youtube_playlist_id = $1
        ORDER BY published_at DESC
        "#,
        project.frontmatter.youtube_playlist
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| ServerError(e.into(), StatusCode::INTERNAL_SERVER_ERROR))
    .map_err(axum::response::IntoResponse::into_response)?;

    Ok(base_constrained(
        html! {
          h1 class="text-3xl" { (project.frontmatter.title) }
          @if let Some(sub) = &project.frontmatter.subtitle {
            h2 class="text-xl mb-4" { (sub) }
          }
          div class="flex flex-row pb-8 align-middle" {
            (StatusTag(project.frontmatter.status))
            a href=(&project.frontmatter.repo) target="_blank" rel="noopener noreferrer" class="mx-2 py-3" {
              i class="fa-brands fa-github" {}
            }
          }

          (markdown)

          h3 class="text-lg mt-8" { "Videos" }
          // (http_server::pages::streams::StreamPostList(streams))
          (http_server::pages::videos::VideoList(youtube_videos))
        },
        OpenGraph {
            title: project.frontmatter.title.clone(),
            description: project.frontmatter.subtitle.clone(),
            ..Default::default()
        },
    ))
}
