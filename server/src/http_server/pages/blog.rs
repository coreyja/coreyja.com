use axum::{extract::Path, http::StatusCode};

use maud::{html, Markup};

use crate::{
    blog::{BlogPostPath, BlogPosts},
    http_server::{pages::blog::md::IntoHtml, templates::base},
};

mod md;

pub async fn posts_index() -> Result<Markup, StatusCode> {
    let posts = BlogPosts::from_static_dir().expect("Failed to load blog posts");
    let posts = posts.posts();

    Ok(base(html! {
      ul {
          @for post in posts {
              li {
                a href=(format!("/posts/{}", post.path().to_string_lossy())) { (post.title()) }
              }
          }
      }
    }))
}

pub async fn post_get(Path(mut key): Path<String>) -> Result<Markup, StatusCode> {
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

    let Some(markdown) = path.to_markdown() else {
      return Err(StatusCode::NOT_FOUND);
    };

    Ok(base(html! {
      h1 { (markdown.title) }
      subtitle { (markdown.date) }

      (markdown.ast.into_html())
    }))
}
