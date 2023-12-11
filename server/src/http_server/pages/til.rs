use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use maud::{html, Markup};
use miette::Result;
use posts::til::TilPosts;
use tracing::instrument;

use crate::{
    http_server::{
        errors::MietteError,
        pages::blog::md::IntoHtml,
        templates::{base_constrained, header::OpenGraph, post_templates::TilPostList},
        ResponseResult,
    },
    AppState,
};

use super::blog::MyChannel;

#[instrument(skip_all)]
pub(crate) async fn til_index(
    State(til_posts): State<Arc<TilPosts>>,
) -> Result<Markup, StatusCode> {
    let posts = til_posts.by_recency();

    Ok(base_constrained(
        html! {
          h1 class="text-3xl" { "Today I Learned" }
          (TilPostList(posts))
        },
        Default::default(),
    ))
}

#[instrument(skip_all)]
pub(crate) async fn rss_feed(
    State(state): State<AppState>,
    State(posts): State<Arc<TilPosts>>,
) -> ResponseResult {
    let channel = MyChannel::from_posts(
        &state.app,
        &state.markdown_to_html_context,
        &posts.by_recency(),
    )?;

    Ok(channel.into_response())
}

#[instrument(skip(til_posts, state))]
pub(crate) async fn til_get(
    State(til_posts): State<Arc<TilPosts>>,
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ResponseResult<Markup> {
    let tils = &til_posts.posts;

    let til = tils
        .iter()
        .find(|p| p.frontmatter.slug == slug)
        .ok_or_else(|| {
            MietteError(
                miette::miette!("No such post found"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    let markdown = til.markdown();
    Ok(base_constrained(
        html! {
          h1 class="text-2xl" { (markdown.title) }
          subtitle class="block text-lg text-subtitle mb-8 " { (markdown.date) }

          div {
            (markdown.ast.into_html(&state.app, &state.markdown_to_html_context)?)
          }
        },
        OpenGraph {
            title: markdown.title.clone(),
            ..Default::default()
        },
    ))
}
