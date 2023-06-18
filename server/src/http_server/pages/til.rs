use std::sync::Arc;

use axum::extract::{Path, State};

use maud::{html, Markup};
use miette::Result;
use reqwest::StatusCode;
use tracing::instrument;

use crate::{
    http_server::{pages::blog::md::IntoHtml, templates::base},
    posts::til::TilPosts,
};

use super::blog::md::HtmlRenderContext;

#[instrument(skip_all)]
pub(crate) async fn til_index(
    State(til_posts): State<Arc<TilPosts>>,
) -> Result<Markup, StatusCode> {
    let mut posts: Vec<_> = til_posts.posts.to_vec();
    posts.sort_by_key(|p| p.frontmatter.date.clone());
    posts.reverse();

    Ok(base(html! {
      h1 class="text-3xl" { "Today I Learned" }
      ul {
        @for post in posts {
          li class="my-4" {
            a href=(format!("/til/{}", post.frontmatter.slug)) {
                span class="text-subtitle text-sm inline-block w-[80px]" { (post.frontmatter.date) }
                " "

                (post.frontmatter.title)
            }
          }
        }
      }
    }))
}

#[instrument(skip(til_posts, context))]
pub(crate) async fn til_get(
    State(til_posts): State<Arc<TilPosts>>,
    State(context): State<HtmlRenderContext>,
    Path(slug): Path<String>,
) -> Result<Markup, StatusCode> {
    let tils = &til_posts.posts;

    let til = tils
        .iter()
        .find(|p| p.frontmatter.slug == slug)
        .ok_or(StatusCode::NOT_FOUND)?;

    let markdown = til.markdown();
    Ok(base(html! {
      h1 class="text-2xl" { (markdown.title) }
      subtitle class="block text-lg text-subtitle mb-8 " { (markdown.date) }

      div {
        (markdown.ast.into_html(&context))
      }
    }))
}
