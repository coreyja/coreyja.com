use maud::{html, Markup, Render};

use crate::posts::til::TilPost;

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
