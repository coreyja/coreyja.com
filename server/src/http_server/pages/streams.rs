use std::sync::Arc;

use axum::extract::State;
use maud::{html, Markup, Render};
use reqwest::StatusCode;
use tracing::instrument;

use crate::{
    http_server::templates::base_constrained,
    posts::{
        blog::LinkTo,
        past_streams::{PastStream, PastStreams},
    },
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
