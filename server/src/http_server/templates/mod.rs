use maud::{html, Markup, PreEscaped, Render};

const LOGO_SVG: &str = include_str!("../../../static/logo.svg");
const LOGO_MONOCHROME_SVG: &str = include_str!("../../../static/logo-monochrome.svg");

const MAX_WIDTH_CONTAINER_CLASSES: &str = "max-w-5xl m-auto px-4";

pub fn head() -> Markup {
    let temporary_remove_service_worker_script = r#"
      navigator.serviceWorker.getRegistrations().then(function(registrations) {
        for(let registration of registrations) {
            registration.unregister();
        } 
      });
      "#;
    html! {
      head {
        title { "coreyja.com" }
        link rel="stylesheet" href="/styles/tailwind.css" {}
        link rel="stylesheet" href="/styles/syntax.css" {}

        link rel="preconnect" href="https://fonts.googleapis.com" {}
        link rel="preconnect" href="https://fonts.gstatic.com" crossorigin {}
        link href="https://fonts.googleapis.com/css2?family=Quicksand:wght@300;400;500;600;700&&display=swap" rel="stylesheet" {}

        link rel="stylesheet" href="https://kit.fontawesome.com/d4a1ffb2a0.css" crossorigin="anonymous";

        meta name="viewport" content="width=device-width, initial-scale=1";

        script type="text/javascript" {
          (PreEscaped(temporary_remove_service_worker_script))
        }
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
      div class="flex flex-col sm:flex-row justify-center items-stretch" {
        div class="flex flex-grow justify-center" {
          div class="max-w-[85%] sm:max-w-lg min-w-[200px] py-8 sm:py-24 flex-grow" {
            a href="/" {
              (PreEscaped(LOGO_SVG))
            }
          }
        }

        nav class="flex flex-grow w-full sm:ml-16 sm:max-w-[50%] pb-16 sm:pb-0" {
          ul class="flex flex-row justify-center sm:justify-end items-center flex-grow" {
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

      body class="
      bg-background
      text-text
      font-sans
      min-h-screen
      flex
      flex-col
      " {
        div ."w-full ".(MAX_WIDTH_CONTAINER_CLASSES) {
          (header())

          (inner)
        }

        (footer())
      }
    }
}

pub fn footer() -> Markup {
    html! {
      div ."flex-grow" {}
      div ."min-h-[100px] bg-subtitle mt-24" {
        div ."flex ".(MAX_WIDTH_CONTAINER_CLASSES) {
          div class="max-w-[10rem] sm:max-w-[15rem] min-w-[100px] py-8 flex-grow" {
            a href="/" {
              (PreEscaped(LOGO_MONOCHROME_SVG))
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

            a href="/posts/rss.xml" target="_blank" rel="noopener noreferrer" {
              i class="fa-solid fa-rss" {}
            }
          }
        }
      }
    }
}

pub(crate) mod buttons;

pub(crate) mod posts;
