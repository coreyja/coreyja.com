use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};

use maud::{html, Markup};
use posts::{
    blog::{BlogPostPath, BlogPosts, MatchesPath, ToCanonicalPath},
    date::PostedOn,
    til::TilPosts,
    Post,
};
use rss::validation::Validate;
use tracing::instrument;

pub(crate) mod md;

use crate::{
    http_server::{
        errors::ServerError,
        pages::blog::md::{
            html::MarkdownRenderContext, FindCoverPhoto, IntoHtml, SyntaxHighlightingContext,
        },
        templates::{base_constrained, header::OpenGraph, post_templates::BlogPostList, ShortDesc},
        LinkTo, ToRssItem,
    },
    AppConfig, AppState,
};

pub(crate) struct MyChannel(rss::Channel);

impl MyChannel {
    #[instrument(skip_all)]
    pub fn from_posts<T>(
        config: &AppConfig,
        context: &SyntaxHighlightingContext,
        posts: &[&Post<T>],
    ) -> cja::Result<Self>
    where
        Post<T>: ToRssItem,
    {
        let items: cja::Result<Vec<rss::Item>> = posts
            .iter()
            .map(|p| p.to_rss_item(config, context))
            .collect();

        Ok(Self::from_items(config, context, &items?))
    }

    pub fn from_items(
        config: &AppConfig,
        _context: &SyntaxHighlightingContext,
        items: &[rss::Item],
    ) -> Self {
        use rss::ChannelBuilder;

        let channel = ChannelBuilder::default()
            .title("coreyja Blog".to_string())
            .link(config.home_page())
            .copyright(Some("Copyright Corey Alexander".to_string()))
            .language(Some("en-us".to_string()))
            .items(items)
            .build();

        Self(channel)
    }

    pub fn validate(&self) -> Result<(), rss::validation::ValidationError> {
        self.0.validate()
    }
}

#[instrument(skip_all)]
pub(crate) async fn rss_feed(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
    let channel = MyChannel::from_posts(
        &state.app,
        &state.syntax_highlighting_context,
        &state.blog_posts.by_recency(),
    )?;

    Ok(channel.into_response())
}

#[instrument(skip_all)]
pub(crate) async fn full_rss_feed(
    State(state): State<AppState>,
    State(blog_posts): State<Arc<BlogPosts>>,
    State(til_posts): State<Arc<TilPosts>>,
) -> Result<impl IntoResponse, ServerError> {
    let mut items_with_date: Vec<(chrono::NaiveDate, rss::Item)> = vec![];
    for p in blog_posts.by_recency() {
        items_with_date.push((
            p.posted_on(),
            p.to_rss_item(&state.app, &state.syntax_highlighting_context)?,
        ));
    }
    for p in til_posts.by_recency() {
        items_with_date.push((
            p.posted_on(),
            p.to_rss_item(&state.app, &state.syntax_highlighting_context)?,
        ));
    }

    items_with_date.sort_by_key(|&(date, _)| std::cmp::Reverse(date));

    let items: Vec<rss::Item> = items_with_date.into_iter().map(|(_, i)| i).collect();

    let channel = MyChannel::from_items(&state.app, &state.syntax_highlighting_context, &items);

    Ok(channel.into_response())
}

impl IntoResponse for MyChannel {
    fn into_response(self) -> Response {
        let r = Response::builder()
            .header("Content-Type", "application/rss+xml")
            .body(self.0.to_string());

        if let Ok(r) = r {
            r.into_response()
        } else {
            let e: ServerError =
                cja::color_eyre::eyre::eyre!("Failed to build RSS Feed response").into();
            e.into_response()
        }
    }
}

#[instrument(skip_all)]
pub(crate) async fn posts_index(State(posts): State<Arc<BlogPosts>>) -> Result<Markup, StatusCode> {
    Ok(base_constrained(
        html! {
          h1 class="text-3xl" { "Blog Posts" }
          (BlogPostList(posts.by_recency()))
        },
        OpenGraph::default(),
    ))
}

#[instrument(skip(state, posts))]
pub(crate) async fn post_get(
    State(state): State<AppState>,
    State(posts): State<Arc<BlogPosts>>,
    Path(key): Path<String>,
) -> Result<Response, StatusCode> {
    {
        let path = BlogPostPath::new(key.clone());
        if path.file_exists() && !path.file_is_markdown() {
            return Ok(path.raw_bytes().into_response());
        }
    }

    let (post, m) = posts
        .posts()
        .iter()
        .find_map(|p| p.matches_path(&key).map(|m| (p, m)))
        .ok_or(StatusCode::NOT_FOUND)?;

    if let MatchesPath::RedirectToCanonicalPath = m {
        return Ok(
            Redirect::permanent(&format!("/posts/{}", post.path.canonical_path())).into_response(),
        );
    }

    let markdown = post.markdown();
    let cover_photo = markdown.ast.0.cover_photo();

    let context = MarkdownRenderContext {
        syntax_highlighting: state.syntax_highlighting_context.clone(),
        current_article_path: post.relative_link(),
    };

    let html = match markdown.ast.into_html(&state.app, &context) {
        Ok(html) => html,
        Err(e) => {
            let server_error = ServerError(e, StatusCode::INTERNAL_SERVER_ERROR);
            tracing::error!(?server_error, "Failed to render markdown to html");

            return Err(server_error.1);
        }
    };

    let image_defaulted_open_graph = match cover_photo {
        Some(cover_photo) => OpenGraph {
            image: Some(cover_photo),
            ..OpenGraph::default()
        },
        None => OpenGraph::default(),
    };

    Ok(base_constrained(
        html! {
          h1 class="text-2xl" { (post.markdown().title) }
          subtitle class="block text-lg text-subtitle mb-8" { (post.markdown().date) }

          div {
            (html)
          }
        },
        OpenGraph {
            title: post.markdown().title,
            r#type: "article".to_string(),
            description: post.short_description(),
            ..image_defaulted_open_graph
        },
    )
    .into_response())
}
