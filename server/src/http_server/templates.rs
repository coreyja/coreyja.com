use maud::{html, Markup, PreEscaped};

const LOGO_SVG: &str = include_str!("../../static/logo.svg");

pub fn head() -> Markup {
    html! {
      head {
        title { "coreyja.com" }
        link rel="stylesheet" href="/styles/tailwind.css" {}
      }
    }
}

pub fn header() -> Markup {
    html! {
      div class="flex px-8" {
        div class="max-w-md mx-auto p-4 flex-grow" {
          (PreEscaped(LOGO_SVG))
        }

        nav class="flex flex-grow justify-end" {
          ul class="grid grid-flow-col items-center gap-16" {
            li {
              a href="/" { "Home" }
            }

            li {
              a href="/posts" { "Posts" }
            }

            li {
              a href="/projects" { "Projects" }
            }

            li {
              a href="/streaming" { "Streaming" }
            }

            li {
              a href="/contact" { "Contact" }
            }
          }
        }
      }
    }
}

pub fn base(inner: Markup) -> Markup {
    html! {
      (head())

      body class="bg-background text-white" {
        (header())

        (inner)
      }
    }
}
