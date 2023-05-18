use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};

use maud::{html, Markup};

use crate::{
    blog::{BlogPostPath, BlogPosts, ToCanonicalPath},
    http_server::{pages::blog::md::IntoHtml, templates::base},
};

use self::md::HtmlRenderContext;

pub(crate) mod md;

pub async fn posts_index() -> Result<Markup, StatusCode> {
    let posts = BlogPosts::from_static_dir().expect("Failed to load blog posts");
    let mut posts: Vec<_> = posts.posts().to_vec();

    posts.sort_by_key(|p| *p.date());
    posts.reverse();

    Ok(base(html! {
      ul {
          @for post in posts {
              li class="my-4" {
                a href=(format!("/posts/{}", post.canonical_path())) {
                    span class="text-subtitle text-sm inline-block w-[80px]" { (post.date()) }
                    " "

                    (post.title())
                }
              }
          }
      }
    }))
}

pub(crate) async fn post_get(
    State(context): State<HtmlRenderContext>,
    Path(key): Path<String>,
) -> Result<Response, StatusCode> {
    {
        let path = BlogPostPath::new(key.clone());
        if path.file_exists() && !path.file_is_markdown() {
            return Ok(path.raw_bytes().into_response());
        }
    }

    let posts = BlogPosts::from_static_dir().expect("Failed to load blog posts");
    let (post, m) = posts
        .posts()
        .iter()
        .find_map(|p| p.matches_path(&key).map(|m| (p, m)))
        .ok_or(StatusCode::NOT_FOUND)?;

    if let crate::blog::MatchesPath::RedirectToCanonicalPath = m {
        return Ok(
            Redirect::permanent(&format!("/posts/{}", post.canonical_path())).into_response(),
        );
    }

    let markdown = post.markdown();
    Ok(base(html! {
      h1 class="text-2xl" { (markdown.title) }
      subtitle class="block text-lg text-subtitle mb-8" { (markdown.date) }

      div {
        (markdown.ast.into_html(&context))
      }
    })
    .into_response())
}
