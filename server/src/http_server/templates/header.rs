use maud::{html, Markup, PreEscaped, Render};

use crate::http_server::templates::LOGO_SVG;

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
      div class="flex flex-col lg:flex-row justify-center items-stretch" {
        div class="flex flex-grow justify-center" {
          div class="max-w-md min-w-[200px] py-8 lg:py-24 flex-grow" {
            a href="/" {
              (PreEscaped(LOGO_SVG))
            }
          }
        }

        nav class="flex flex-grow w-full lg:ml-16 lg:max-w-[50%] pb-16 lg:pb-0" {
          ul class="flex flex-row justify-center lg:justify-end items-center flex-grow" {
            (HeaderLink { href: "/", text: "Home" })
            (HeaderLink { href: "/posts", text: "Posts" })
            (HeaderLink { href: "/til", text: "TILs" })
            (HeaderLink { href: "/newsletter", text: "Newsletter" })
          }
        }
      }
    }
}
