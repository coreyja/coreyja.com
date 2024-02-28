use maud::{html, Markup, PreEscaped};

use crate::http_server::templates::{
    LOGO_FLAT_SVG, LOGO_MONOCHROME_SVG, LOGO_SVG, MAX_WIDTH_CONTAINER_CLASSES,
};

pub fn newsletter_signup_footer() -> Markup {
    html! {
      div ."bg-[rgba(178,132,255,0.1)] py-16 flex flex-col items-center space-y-8 px-4" {
        h2 ."text-3xl leading-none" { "coreyja weekly" }
        p ."max-x-prose leading-loose" {
          "My weekly newsletter tailored at developers who are eager to grow with me!"
          br;
          "Every week will be unique, but expect topics focusing around Web Development and Rust"
        }

        form
          action="https://app.convertkit.com/forms/5312462/subscriptions"
          method="post"
          class="w-full max-w-md flex flex-row gap-4"
          {
            input
              type="email"
              name="email_address"
              class="flex-grow py-2 px-2 rounded-md text-grey-999"
              placeholder="Enter your email address"
              required="required"
              ;

            input
              type="submit"
              value="Subscribe"
              class="bg-berryBlue rounded-lg px-8 py-2"
              ;
        }
      }
    }
}

pub fn footer() -> Markup {
    html! {
      div class="flex-grow mb-24" {}
      (newsletter_signup_footer())
      div ."min-h-[100px] bg-footer" {
        div ."flex ".(MAX_WIDTH_CONTAINER_CLASSES) {
          div class="max-w-[10rem] sm:max-w-[15rem] min-w-[100px] py-8 flex-grow" {
            a href="/" {
              (PreEscaped(LOGO_FLAT_SVG))
            }
          }

          div ."flex-grow" {}

          ul class="flex flex-row items-center text-background space-x-4 sm:space-x-8 text-xl sm:text-2xl" {
            a href="https://github.com/coreyja" target="_blank" rel="noopener noreferrer" {
              i class="fa-brands fa-github" {}
            }

            a href="https://twitch.tv/coreyja" target="_blank" rel="noopener noreferrer" {
              i class="fa-brands fa-twitch" {}
            }

            a href="https://youtube.com/@coreyja" target="_blank" rel="noopener noreferrer" {
              i class="fa-brands fa-youtube" {}
            }

            a href="https://toot.cat/@coreyja" target="_blank" rel="noopener noreferrer" {
              i class="fa-brands fa-mastodon" {}
            }

            a href="/rss.xml" target="_blank" rel="noopener noreferrer" {
              i class="fa-solid fa-rss" {}
            }
          }
        }
      }
    }
}
