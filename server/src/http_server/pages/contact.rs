use maud::{html, Markup};

use crate::http_server::templates::{base_constrained, header::OpenGraph};

pub(crate) async fn contact() -> Markup {
    base_constrained(
        html! {
            h1 class="my-4 text-2xl" {
                "Contact Me"
            }

            p class="my-4" {
              "Feel free to reach out to me with any questions, comments, or suggestions!"
              br;
              "I'll do my best to respond to all messages, and love to hear from anyone checking out my site!"
            }

            p class="my-4" {
              "You can email me at "
              a class="underline" href="mailto:contact@coreyja.com" {
                "contact@coreyja.com"
              }
              " or on Bluesky as "
              a class="underline" href="https://bsky.app/profile/coreyja.com" {
                "@coreyja.com"
              }
            }

            p class="my-4" {
              "I'm also on Mastodon at "
              a class="underline" href="https://toot.cat/@coreyja" {
                "@coreyja"
              }
              " but I've been less active there lately."
            }

        },
        OpenGraph::default(),
    )
}
