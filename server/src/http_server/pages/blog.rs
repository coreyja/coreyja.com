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

use crate::{
    http_server::{
        errors::MietteError,
        pages::blog::md::IntoHtml,
        templates::{base_constrained, header::OpenGraph, post_templates::BlogPostList, ShortDesc},
        ResponseResult, ToRssItem,
    },
    AppConfig, AppState,
};

use self::md::SyntaxHighlightingContext;

pub(crate) mod md;

pub(crate) struct MyChannel(rss::Channel);

impl MyChannel {
    #[instrument(skip_all)]
    pub fn from_posts<T>(
        config: &AppConfig,
        context: &SyntaxHighlightingContext,
        posts: &[&Post<T>],
    ) -> miette::Result<Self>
    where
        Post<T>: ToRssItem,
    {
        let items: miette::Result<Vec<rss::Item>> = posts
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
    State(posts): State<Arc<BlogPosts>>,
) -> Result<impl IntoResponse, MietteError> {
    let channel = MyChannel::from_posts(
        &state.app,
        &state.markdown_to_html_context,
        &posts.by_recency(),
    )?;

    Ok(channel.into_response())
}

#[instrument(skip_all)]
pub(crate) async fn full_rss_feed(
    State(state): State<AppState>,
    State(blog_posts): State<Arc<BlogPosts>>,
    State(til_posts): State<Arc<TilPosts>>,
) -> Result<impl IntoResponse, MietteError> {
    let mut items_with_date: Vec<(chrono::NaiveDate, rss::Item)> = vec![];
    for p in blog_posts.by_recency() {
        items_with_date.push((
            p.posted_on(),
            p.to_rss_item(&state.app, &state.markdown_to_html_context)?,
        ));
    }
    for p in til_posts.by_recency() {
        items_with_date.push((
            p.posted_on(),
            p.to_rss_item(&state.app, &state.markdown_to_html_context)?,
        ));
    }

    items_with_date.sort_by_key(|&(date, _)| std::cmp::Reverse(date));

    let items: Vec<rss::Item> = items_with_date.into_iter().map(|(_, i)| i).collect();

    let channel = MyChannel::from_items(&state.app, &state.markdown_to_html_context, &items);

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
            let e: MietteError = miette::miette!("Failed to build RSS Feed response").into();
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
        Default::default(),
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
    let html = match markdown
        .ast
        .into_html(&state.app, &state.markdown_to_html_context)
    {
        Ok(html) => html,
        Err(e) => {
            let miette_error = MietteError(e, StatusCode::INTERNAL_SERVER_ERROR);
            sentry::capture_error(&miette_error);

            return Err(miette_error.1);
        }
    };

    Ok(base_constrained(
        html! {
          h1 class="text-2xl" { (markdown.title) }
          subtitle class="block text-lg text-subtitle mb-8" { (markdown.date) }

          div {
            (html)
          }
        },
        OpenGraph {
            title: markdown.title,
            r#type: "article".to_string(),
            description: post.short_description(),
            ..Default::default()
        },
    )
    .into_response())
}
