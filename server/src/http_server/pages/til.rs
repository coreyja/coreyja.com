use std::sync::Arc;

use axum::extract::State;

use maud::{html, Markup};
use miette::Result;
use reqwest::StatusCode;

use crate::{http_server::templates::base, til::TilPosts};

pub(crate) async fn til_index(
    State(til_posts): State<Arc<TilPosts>>,
) -> Result<Markup, StatusCode> {
    let mut posts: Vec<_> = til_posts.posts.to_vec();
    posts.sort_by_key(|p| p.date);
    posts.reverse();

    Ok(base(html! {
      h1 class="text-3xl" { "Today I Learned" }
      ul {
        @for post in posts {
          li class="my-4" {
            a href=(format!("/til/{}", post.slug)) {
                span class="text-subtitle text-sm inline-block w-[80px]" { (post.date) }
                " "

                (post.title)
            }
          }
        }
      }
    }))
}
