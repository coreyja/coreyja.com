use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};

use maud::{html, Markup};
use tracing::instrument;

use crate::{
    http_server::{
        pages::blog::md::IntoHtml,
        templates::{base_constrained, posts::BlogPostList},
    },
    posts::blog::{BlogPostPath, BlogPosts, MatchesPath, ToCanonicalPath},
    AppConfig,
};

use self::md::HtmlRenderContext;

pub(crate) mod md;

struct MyChannel(rss::Channel);

#[instrument(skip_all)]
pub(crate) async fn rss_feed(
    State(config): State<AppConfig>,
    State(posts): State<Arc<BlogPosts>>,
) -> Result<impl IntoResponse, StatusCode> {
    let channel = generate_rss(config, &posts);
    let channel = MyChannel(channel);

    Ok(channel.into_response())
}

#[instrument(skip_all)]
pub(crate) fn generate_rss(config: AppConfig, posts: &BlogPosts) -> rss::Channel {
    let mut posts = posts.posts().clone();
    posts.sort_by_key(|p| *p.date());
    posts.reverse();

    let items: Vec<_> = posts.iter().map(|p| p.to_rss_item(&config)).collect();

    use rss::ChannelBuilder;

    let channel = ChannelBuilder::default()
        .title("coreyja Blog".to_string())
        .link(config.home_page())
        .items(items)
        .build();
    channel
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
    State(context): State<HtmlRenderContext>,
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
