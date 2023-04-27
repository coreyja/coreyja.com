use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use maud::{html, Markup};

use crate::{
    blog::{BlogPostPath, BlogPosts},
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
                    a href=(format!("/posts/{}", post.path().to_string_lossy())) {
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
    Path(mut key): Path<String>,
) -> Result<Response, StatusCode> {
    // TODO: Eventually
    //
    // I think we can move away from the wildcard route and instead
    // use the static-ness of BLOG_DIR to setup all the routes on server
    // boot.
    // Thay way we can static routes to route the different posts and avoid the wildcard
    // This might make it easier to do something like generate a sitemap from the routes
    key = key.strip_suffix('/').unwrap_or(&key).to_string();
    key = key.strip_suffix("index.md").unwrap_or(&key).to_string();

    let mut path = BlogPostPath::new(key.clone());

    if !path.file_exists() {
        path = BlogPostPath::new(format!("{key}.md"));
    }

    if !path.file_exists() {
        path = BlogPostPath::new(format!("{key}/index.md"));
    }

    if !path.file_is_markdown() {
        return Ok(path.raw_bytes().into_response());
    }

    let Some(markdown) = path.to_markdown() else {
      return Err(StatusCode::NOT_FOUND);
    };

    Ok(base(html! {
      h1 class="text-2xl" { (markdown.title) }
      subtitle class="block text-lg text-subtitle mb-8" { (markdown.date) }

      div class="" {
        (markdown.ast.into_html(&context))
      }
    })
    .into_response())
}
