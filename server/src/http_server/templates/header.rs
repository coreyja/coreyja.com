use std::borrow::Borrow;

use maud::{html, Markup, PreEscaped, Render};

use crate::http_server::templates::LOGO_SVG;

pub struct OpenGraph {
    pub title: String,
    pub r#type: String,
    pub image: Option<String>,
    pub video: Option<String>,
    pub url: String,
    pub description: Option<String>,
}

impl Default for OpenGraph {
    fn default() -> Self {
        Self {
            title: "coreyja".to_owned(),
            r#type: "website".to_owned(),
            image: Some("https://coreyja.com/static/opengraph.png".to_owned()),
            video: None,
            url: "coreyja.com".to_owned(),
            description: Some(
                "Corey's personal site that contains all his projects and streams".to_owned(),
            ),
        }
    }
}

impl Render for OpenGraph {
    fn render(&self) -> Markup {
        html! {
          meta property="og:title" content=(self.title) {}
          meta property="og:type" content=(self.r#type) {}
          meta property="og:url" content=(self.url) {}
          @if let Some(description) = &self.description {
            meta property="og:description" content=(description) {}
          }
          @if let Some(image) = &self.image {
            meta property="og:image" content=(image) {}
          }
        }
    }
}

pub fn head(og: impl Borrow<OpenGraph>) -> Markup {
    let temporary_remove_service_worker_script = r"
      navigator.serviceWorker.getRegistrations().then(function(registrations) {
        for(let registration of registrations) {
            registration.unregister();
        } 
      });
      ";

    html! {
      head {
        title { "coreyja.com" }
        link rel="stylesheet" href="/styles/tailwind.css" {}
        link rel="stylesheet" href="/styles/syntax.css" {}
        link rel="stylesheet" href="/styles/comic_code.css" {}

        link rel="preconnect" href="https://fonts.googleapis.com" {}
        link rel="preconnect" href="https://fonts.gstatic.com" crossorigin {}
        link href="https://fonts.googleapis.com/css2?family=Quicksand:wght@300;400;500;600;700&&display=swap" rel="stylesheet" {}

        link rel="stylesheet" href="https://kit.fontawesome.com/d4a1ffb2a0.css" crossorigin="anonymous";

        meta name="viewport" content="width=device-width, initial-scale=1";

        (og.borrow())

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
          li ."sm:mx-8" {
            a href=(self.href) { (self.text) }
          }
        }
    }
}

pub fn header() -> Markup {
    html! {
      div class="flex flex-col justify-center items-stretch" {
        div class="flex flex-grow justify-center" {
          div class="max-w-sm min-w-[200px] py-8 lg:py-12 flex-grow" {
            a href="/" {
              (PreEscaped(LOGO_SVG))
            }
          }
        }

        nav class="flex flex-grow w-full pb-4 sm:pb-8" {
          ul class="flex flex-col sm:flex-row justify-center sm:items-center flex-grow space-y-4 sm:space-y-0" {
            (HeaderLink { href: "/", text: "Home" })
            (HeaderLink { href: "/posts", text: "Posts" })
            (HeaderLink { href: "/til", text: "TILs" })
            (HeaderLink { href: "/videos", text: "Videos" })
            (HeaderLink { href: "/projects", text: "Projects" })
            (HeaderLink { href: "/newsletter", text: "Newsletter" })
          }
        }
      }
    }
}
