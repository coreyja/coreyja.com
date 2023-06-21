use maud::{html, Markup, Render};

use crate::posts::{
    blog::{BlogPost, ToCanonicalPath},
    til::TilPost,
};

pub(crate) struct TilPostList<'a>(pub(crate) Vec<&'a TilPost>);

impl<'a> Render for TilPostList<'a> {
    fn render(&self) -> Markup {
        html! {
          ul {
            @for post in &self.0 {
              li class="my-4" {
                a href=(format!("/til/{}", post.frontmatter.slug)) {
                    span class="text-subtitle text-sm inline-block w-[80px]" { (post.frontmatter.date) }
                    " "

                    (post.frontmatter.title)
                }
              }
            }
          }
        }
    }
}

pub(crate) struct BlogPostList<'a>(pub(crate) Vec<&'a BlogPost>);

impl<'a> Render for BlogPostList<'a> {
    fn render(&self) -> Markup {
        html! {
          ul {
            @for post in &self.0 {
                li class="my-4" {
                  a href=(format!("/posts/{}", post.canonical_path())) {
                      span class="text-subtitle text-sm inline-block w-[80px]" { (post.date()) }
                      " "

                      (post.title())
                  }
                }
            }
        }
        }
    }
}
