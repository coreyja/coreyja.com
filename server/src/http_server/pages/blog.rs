use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};

use maud::{html, Markup};
use rss::validation::Validate;
use tracing::instrument;

use crate::{
    http_server::{
        pages::blog::md::IntoHtml,
        templates::{base_constrained, posts::BlogPostList},
    },
    posts::{
        blog::{BlogPostPath, BlogPosts, MatchesPath, ToCanonicalPath},
        date::PostedOn,
        til::TilPosts,
        Post, ToRssItem,
    },
    AppConfig,
};

use self::md::SyntaxHighlightingContext;

pub(crate) mod md;

pub(crate) struct MyChannel(rss::Channel);

impl MyChannel {
    #[instrument(skip_all)]
    pub fn from_posts<T>(
        config: AppConfig,
        render_context: SyntaxHighlightingContext,
        posts: &[&Post<T>],
    ) -> Self
    where
        Post<T>: ToRssItem,
    {
        let items: Vec<_> = posts
            .iter()
            .map(|p| p.to_rss_item(&config, &render_context))
            .collect();

        Self::from_items(config, &items)
    }

    pub fn from_items(config: AppConfig, items: &[rss::Item]) -> Self {
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
    State(config): State<AppConfig>,
    State(context): State<SyntaxHighlightingContext>,
    State(posts): State<Arc<BlogPosts>>,
) -> Result<impl IntoResponse, StatusCode> {
    let channel = MyChannel::from_posts(config, context, &posts.by_recency());

    Ok(channel.into_response())
}

#[instrument(skip_all)]
pub(crate) async fn full_rss_feed(
    State(config): State<AppConfig>,
    State(context): State<SyntaxHighlightingContext>,
    State(blog_posts): State<Arc<BlogPosts>>,
    State(til_posts): State<Arc<TilPosts>>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut items_with_date: Vec<(chrono::NaiveDate, rss::Item)> = vec![];
    items_with_date.extend(
        blog_posts
            .by_recency()
            .into_iter()
            .map(|p| (p.posted_on(), p.to_rss_item(&config, &context))),
    );
    items_with_date.extend(
        til_posts
            .by_recency()
            .into_iter()
            .map(|p| (p.posted_on(), p.to_rss_item(&config, &context))),
    );
    items_with_date.sort_by_key(|&(date, _)| std::cmp::Reverse(date));

    let items: Vec<rss::Item> = items_with_date.into_iter().map(|(_, i)| i).collect();

    let channel = MyChannel::from_items(config, &items);

    Ok(channel.into_response())
}

impl IntoResponse for MyChannel {
    fn into_response(self) -> Response {
        Response::builder()
            .header("Content-Type", "application/rss+xml")
            .body(self.0.to_string())
            .unwrap()
            .into_response()
    }
}

#[instrument(skip_all)]
pub(crate) async fn posts_index(State(posts): State<Arc<BlogPosts>>) -> Result<Markup, StatusCode> {
    Ok(base_constrained(html! {
      h1 class="text-3xl" { "Blog Posts" }
      (BlogPostList(posts.by_recency()))
    }))
}

#[instrument(skip(context, posts))]
pub(crate) async fn post_get(
    State(context): State<SyntaxHighlightingContext>,
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
            Redirect::permanent(&format!("/posts/{}", post.canonical_path())).into_response(),
        );
    }

    let markdown = post.markdown();
    Ok(base_constrained(html! {
      h1 class="text-2xl" { (markdown.title) }
      subtitle class="block text-lg text-subtitle mb-8" { (markdown.date) }

      div {
        (markdown.ast.into_html(&context))
      }
    })
    .into_response())
}
