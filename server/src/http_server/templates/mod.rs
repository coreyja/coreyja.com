use maud::{html, Markup, PreEscaped};

const LOGO_SVG: &str = include_str!("../../../static/logo.svg");

pub fn head() -> Markup {
    html! {
      head {
        title { "coreyja.com" }
        link rel="stylesheet" href="/styles/tailwind.css" {}

        link rel="preconnect" href="https://fonts.googleapis.com" {}
        link rel="preconnect" href="https://fonts.gstatic.com" crossorigin {}
        link href="https://fonts.googleapis.com/css2?family=Quicksand:wght@300;400;500;600;700&&display=swap" rel="stylesheet" {}
      }
    }
}

pub fn header() -> Markup {
    html! {
      div class="flex" {
        div class="max-w-lg min-w-[200px] py-24 flex-grow" {
          a href="/" {
            (PreEscaped(LOGO_SVG))
          }
        }

        nav class="flex flex-grow justify-end w-full ml-16 max-w-[50%]" {
          ul class="flex flex-row items-center justify-between flex-grow" {
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

      body class="bg-background text-text px-4 max-w-5xl m-auto font-sans" {
        (header())

        (inner)
      }
    }
}

pub(crate) mod buttons;
