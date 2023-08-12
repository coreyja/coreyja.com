use std::sync::Arc;

use axum::extract::{Path, State};
use chrono::NaiveDate;
use maud::{html, Markup, Render};
use posts::past_streams::{PastStream, PastStreams};
use reqwest::StatusCode;
use tracing::instrument;

use crate::{
    http_server::{pages::blog::md::IntoHtml, templates::base_constrained, LinkTo},
    AppState,
};

#[instrument(skip_all)]
pub(crate) async fn streams_index(
    State(steams): State<Arc<PastStreams>>,
) -> Result<Markup, StatusCode> {
    let posts = steams.by_recency();

    Ok(base_constrained(html! {
      h1 class="text-3xl" { "Past Streams" }

      (StreamPostList(posts))
    }))
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
    State(state): State<AppState>,
    Path(date): Path<NaiveDate>,
) -> Result<Markup, StatusCode> {
    let til = streams
        .streams
        .iter()
        .find(|p| p.frontmatter.date == date)
        .ok_or(StatusCode::NOT_FOUND)?;

    let markdown = til.markdown();

    let youtube_embed_url = til.frontmatter.youtube_url.as_ref().map(|url| {
        let parts = url.split('/').collect::<Vec<_>>();
        let video_id = parts.last().unwrap();
        format!("https://www.youtube.com/embed/{}", video_id)
    });
    Ok(base_constrained(html! {
      h1 class="text-2xl" { (markdown.title) }
      subtitle class="block text-lg text-subtitle mb-8 " { (markdown.date) }

      @if let Some(url) = youtube_embed_url {
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
        (markdown.ast.into_html(&state))
      }
    }))
}
