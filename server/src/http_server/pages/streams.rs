use std::sync::Arc;

use axum::extract::{Path, State};
use chrono::NaiveDate;
use maud::{html, Markup, Render};
use miette::Result;
use posts::{
    past_streams::{PastStream, PastStreams},
    projects::Projects,
};
use reqwest::StatusCode;
use tracing::instrument;

use crate::{
    http_server::{
        errors::MietteError,
        pages::blog::md::IntoHtml,
        templates::{base_constrained, header::OpenGraph},
        LinkTo,
    },
    AppState,
};

#[instrument(skip_all)]
pub(crate) async fn streams_index(
    State(steams): State<Arc<PastStreams>>,
) -> Result<Markup, StatusCode> {
    let posts = steams.by_recency();

    Ok(base_constrained(
        html! {
          h1 class="text-3xl" { "Past Streams" }

          (StreamPostList(posts))
        },
        Default::default(),
    ))
}
pub(crate) struct StreamPostList<'a>(pub(crate) Vec<&'a PastStream>);

impl<'a> Render for StreamPostList<'a> {
    fn render(&self) -> Markup {
        html! {
          ul {
            @for post in &self.0 {
              li class="my-4" {
                a href=(post.relative_link()) {
                    span class="text-subtitle text-sm inline-block w-[80px]" { (post.frontmatter.date) }
                    " "

                    (post.frontmatter.title)
                }
              }
            }
          }
        }
    }
}

#[instrument(skip(streams, state))]
pub(crate) async fn stream_get(
    State(streams): State<Arc<PastStreams>>,
    State(projects): State<Arc<Projects>>,
    State(state): State<AppState>,
    Path(date): Path<NaiveDate>,
) -> Result<Markup, MietteError> {
    let til = streams
        .streams
        .iter()
        .find(|p| p.frontmatter.date == date)
        .ok_or_else(|| MietteError(miette::miette!("Stream not found"), StatusCode::NOT_FOUND))?;

    let project = til
        .frontmatter
        .project
        .as_ref()
        .and_then(|slug| projects.projects.iter().find(|p| p.slug().unwrap() == slug));

    let markdown = til.markdown();

    let youtube_embed_url: Option<Result<String>> =
        til.frontmatter.youtube_url.as_ref().map(|url| {
            let parts = url.split('/').collect::<Vec<_>>();
            let video_id = parts
                .last()
                .ok_or_else(|| miette::miette!("Failed to parse YouTube URL",))?;
            Ok(format!("https://www.youtube.com/embed/{}", video_id))
        });

    let youtube_embed_url = youtube_embed_url.transpose()?;
    Ok(base_constrained(
        html! {
          h1 class="text-2xl" { (markdown.title) }
          subtitle class="block text-lg text-subtitle mb-8 " { (markdown.date) }

          @if let Some(project) = project {
            div class="mb-8" {
              "Project: "
              a href=(project.relative_link()?) { (project.frontmatter.title) }
            }
          }

          @if let Some(url) = youtube_embed_url.clone() {
            iframe
              id="ytplayer"
              type="text/html"
              width="640"
              height="360"
              src=(url)
              frameborder="0"
              {}
          }

          div {
            (markdown.ast.into_html(&state.app, &state.markdown_to_html_context)?)
          }
        },
        OpenGraph {
            title: markdown.title.clone(),
            video: youtube_embed_url,
            ..Default::default()
        },
    ))
}
