use maud::{html, Markup, PreEscaped, Render};

const LOGO_SVG: &str = include_str!("../../../static/logo.svg");
const LOGO_MONOCHROME_SVG: &str = include_str!("../../../static/logo-monochrome.svg");

pub fn head() -> Markup {
    html! {
      head {
        title { "coreyja.com" }
        link rel="stylesheet" href="/styles/tailwind.css" {}
        link rel="stylesheet" href="/styles/syntax.css" {}

        link rel="preconnect" href="https://fonts.googleapis.com" {}
        link rel="preconnect" href="https://fonts.gstatic.com" crossorigin {}
        link href="https://fonts.googleapis.com/css2?family=Quicksand:wght@300;400;500;600;700&&display=swap" rel="stylesheet" {}

        link rel="stylesheet" href="https://kit.fontawesome.com/d4a1ffb2a0.css" crossorigin="anonymous"

        meta name="viewport" content="width=device-width, initial-scale=1";
      }
    }
}

struct HeaderLink {
    href: &'static str,
    text: &'static str,
}

impl Render for HeaderLink {
    fn render(&self) -> Markup {
        html! {
          li ."mx-8" {
            a href=(self.href) { (self.text) }
          }
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
          ul class="flex flex-row items-center flex-grow" {
            (HeaderLink { href: "/", text: "Home" })
            (HeaderLink { href: "/posts", text: "Posts" })
            (HeaderLink { href: "/til", text: "TILs" })
          }
        }
      }
    }
}

pub fn base(inner: Markup) -> Markup {
    html! {
      (head())

      body class="bg-background text-text px-4 max-w-5xl m-auto font-sans min-h-screen flex flex-col" {
        (header())

        (inner)

        (footer())
      }
    }
}

pub fn footer() -> Markup {
    html! {
      div ."flex-grow" {}
      div ."min-h-[100px] bg-subtitle flex -mx-[100%] px-[100%] flex justify-between mt-24" {
        div class="max-w-[15rem] min-w-[100px] py-8 flex-grow" {
          a href="/" {
            (PreEscaped(LOGO_MONOCHROME_SVG))
          }
        }

        ul class="flex flex-row items-center text-background space-x-8 text-2xl" {
          a href="https://github.com/coreyja" target="_blank" rel="noopener noreferrer" {
            i class="fa-brands fa-github" {}
          }

          a href="https://twitch.tv/coreyja" target="_blank" rel="noopener noreferrer" {
            i class="fa-brands fa-twitch" {}
          }

          a href="https://toot.cat/@coreyja" target="_blank" rel="noopener noreferrer" {
            i class="fa-brands fa-mastodon" {}
          }

          a href="/posts/rss.xml" target="_blank" rel="noopener noreferrer" {
            i class="fa-solid fa-rss" {}
          }
        }
      }
    }
}

pub(crate) mod buttons;

pub(crate) mod posts;
