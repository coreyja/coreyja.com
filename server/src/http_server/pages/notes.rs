use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use cja::Result;
use maud::{html, Markup};
use posts::notes::NotePosts;
use tracing::instrument;

use crate::{
    bsky::fetch_thread,
    http_server::{
        errors::ServerError,
        pages::blog::md::html::{IntoHtml, MarkdownRenderContext},
        templates::{base_constrained, header::OpenGraph, post_templates::NotePostList, ShortDesc},
        LinkTo, ResponseResult,
    },
    AppState,
};

use super::blog::MyChannel;

#[instrument(skip_all)]
pub(crate) async fn notes_index(
    State(state): State<AppState>,
    State(note_posts): State<Arc<NotePosts>>,
) -> Result<Markup, StatusCode> {
    let posts = note_posts.by_recency();

    Ok(base_constrained(
        html! {
          h1 class="text-3xl" { "Notes" }
          (NotePostList(posts))
        },
        OpenGraph::default_for_path(&state.app, "/notes"),
    ))
}

#[instrument(skip_all)]
pub(crate) async fn rss_feed(
    State(state): State<AppState>,
    State(posts): State<Arc<NotePosts>>,
) -> ResponseResult {
    let channel = MyChannel::from_posts(
        &state.app,
        &state.syntax_highlighting_context,
        &posts.by_recency(),
    )?;

    Ok(channel.into_response())
}

#[instrument(skip(note_posts, state))]
pub(crate) async fn notes_get(
    State(note_posts): State<Arc<NotePosts>>,
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ResponseResult<Markup> {
    let notes = &note_posts.posts;

    let note = notes
        .iter()
        .find(|p| p.frontmatter.slug == slug)
        .ok_or_else(|| {
            ServerError(
                cja::color_eyre::eyre::eyre!("No such note found"),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    let markdown = note.markdown();

    let bsky_thread = if let Some(bsky_post_url) = &note.frontmatter.bsky_url {
        match fetch_thread(bsky_post_url).await {
            Ok(thread) => Some((bsky_post_url.as_str(), thread)),
            Err(e) => {
                tracing::warn!(?e, "Failed to fetch Bluesky thread for note");
                None
            }
        }
    } else {
        None
    };

    let card_route_path = format!("/og/notes/{}.svg", note.frontmatter.slug);
    let og_image = crate::http_server::templates::og::og_image_url(&state.app, &card_route_path);

    let canonical_url = state
        .app
        .app_url(&format!("/notes/{}", note.frontmatter.slug));

    let title = markdown.title.clone();
    let published_time = note
        .frontmatter
        .date
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .to_rfc3339();

    Ok(base_constrained(
        html! {
          h1 class="text-2xl" { (markdown.title) }
          subtitle class="block text-lg text-subtitle mb-8 " { (markdown.date) }

          div {
            (markdown.ast.into_html(&state.app, &MarkdownRenderContext { syntax_highlighting: state.syntax_highlighting_context.clone(), current_article_path: note.relative_link() })?)
          }

          @if let Some((bsky_url, thread)) = bsky_thread {
            div class="mt-8" {
              (super::blog::bluesky_post_stats(bsky_url, &thread))
              (super::blog::bsky_comments(bsky_url, thread))
            }
          }
        },
        OpenGraph {
            title: title.clone(),
            r#type: "article".to_string(),
            description: note.short_description(),
            image: Some(og_image),
            image_width: Some(1200),
            image_height: Some(630),
            image_alt: Some(title),
            url: canonical_url,
            site_name: Some("coreyja".to_string()),
            locale: Some("en_US".to_string()),
            twitter_site: Some("@coreyja.com".to_string()),
            twitter_card: None,
            published_time: Some(published_time),
            author: Some("Corey Alexander".to_string()),
            tags: note.frontmatter.tags.clone(),
            ..Default::default()
        },
    ))
}
