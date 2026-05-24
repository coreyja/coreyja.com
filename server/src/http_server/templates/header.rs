use std::borrow::Borrow;

use maud::{html, Markup, PreEscaped, Render};

use crate::http_server::templates::LOGO_DARK_FLAT_SVG;
use crate::AppConfig;

pub struct OpenGraph {
    pub title: String,
    pub r#type: String,
    pub image: Option<String>,
    pub video: Option<String>,
    pub url: String,
    pub description: Option<String>,
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
    pub image_alt: Option<String>,
    pub site_name: Option<String>,
    pub locale: Option<String>,
    pub twitter_site: Option<String>,
    pub twitter_card: Option<String>,
    /// RFC3339 timestamp, only meaningful for `type=article`.
    pub published_time: Option<String>,
    pub author: Option<String>,
    /// Emits `article:tag` once per entry.
    pub tags: Vec<String>,
}

impl Default for OpenGraph {
    fn default() -> Self {
        Self {
            title: "coreyja".to_owned(),
            r#type: "website".to_owned(),
            image: Some("https://coreyja.com/static/opengraph.png".to_owned()),
            video: None,
            url: String::new(),
            description: Some(
                "Corey's personal site that contains all his projects and streams".to_owned(),
            ),
            image_width: None,
            image_height: None,
            image_alt: None,
            site_name: None,
            locale: None,
            twitter_site: None,
            twitter_card: None,
            published_time: None,
            author: None,
            tags: Vec::new(),
        }
    }
}

impl OpenGraph {
    /// Default `OpenGraph` with `url` populated to the site root from `config`.
    pub fn default_for(config: &AppConfig) -> Self {
        Self {
            url: config.app_url("/"),
            ..Self::default()
        }
    }
}

impl Render for OpenGraph {
    fn render(&self) -> Markup {
        let effective_twitter_card = self.twitter_card.clone().or_else(|| {
            if self.image.is_some() {
                Some("summary_large_image".to_string())
            } else {
                None
            }
        });

        html! {
          meta property="og:title" content=(self.title) {}
          meta property="og:type" content=(self.r#type) {}
          @if !self.url.is_empty() {
            meta property="og:url" content=(self.url) {}
          }
          @if let Some(description) = &self.description {
            meta property="og:description" content=(description) {}
          }
          @if let Some(image) = &self.image {
            meta property="og:image" content=(image) {}
          }
          @if let Some(w) = self.image_width {
            meta property="og:image:width" content=(w) {}
          }
          @if let Some(h) = self.image_height {
            meta property="og:image:height" content=(h) {}
          }
          @if let Some(alt) = &self.image_alt {
            meta property="og:image:alt" content=(alt) {}
          }
          @if let Some(site_name) = &self.site_name {
            meta property="og:site_name" content=(site_name) {}
          }
          @if let Some(locale) = &self.locale {
            meta property="og:locale" content=(locale) {}
          }
          @if let Some(published_time) = &self.published_time {
            meta property="article:published_time" content=(published_time) {}
          }
          @if let Some(author) = &self.author {
            meta property="article:author" content=(author) {}
          }
          @for tag in &self.tags {
            meta property="article:tag" content=(tag) {}
          }
          @if let Some(card) = &effective_twitter_card {
            meta name="twitter:card" content=(card) {}
          }
          @if let Some(twitter_site) = &self.twitter_site {
            meta name="twitter:site" content=(twitter_site) {}
          }
          @if self.image.is_some() {
            meta name="twitter:title" content=(self.title) {}
            @if let Some(description) = &self.description {
              meta name="twitter:description" content=(description) {}
            }
            @if let Some(image) = &self.image {
              meta name="twitter:image" content=(image) {}
            }
          }
        }
    }
}

pub fn head(og: impl Borrow<OpenGraph>) -> Markup {
    html! {
      head {
        title { "coreyja.com" }
        link rel="stylesheet" href="/styles/tailwind.css" {}
        link rel="stylesheet" href="/styles/syntax.css" {}
        link rel="stylesheet" href="/styles/comic_code.css" {}

        link rel="stylesheet" href="/static/view-transitions.css" {}
        script src="/static/view-transitions.js" defer {}

        link rel="preconnect" href="https://fonts.googleapis.com" {}
        link rel="preconnect" href="https://fonts.gstatic.com" crossorigin {}
        link href="https://fonts.googleapis.com/css2?family=Quicksand:wght@300;400;500;600;700&&display=block" rel="stylesheet" {}

        link rel="stylesheet" href="https://kit.fontawesome.com/d4a1ffb2a0.css" crossorigin="anonymous";

        meta name="viewport" content="width=device-width, initial-scale=1";

        link rel="apple-touch-icon" sizes="180x180" href="/static/icons/apple-touch-icon.png";
        link rel="icon" type="image/png" sizes="32x32" href="/static/icons/favicon-32x32.png";
        link rel="icon" type="image/png" sizes="16x16" href="/static/icons/favicon-16x16.png";
        link rel="manifest" href="/static/icons/site.webmanifest";
        link rel="mask-icon" href="/static/icons/safari-pinned-tab.svg" color="#401f74";
        link rel="shortcut icon" href="/static/icons/favicon.ico";
        meta name="msapplication-TileColor" content="#603cba";
        meta name="msapplication-config" content="/static/icons/browserconfig.xml";
        meta name="theme-color" content="#401f74";

        (og.borrow())
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
          li ."mx-4 sm:mx-8 my-4" {
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
              (PreEscaped(LOGO_DARK_FLAT_SVG))
            }
          }
        }

        nav class="flex flex-grow w-full pb-4 sm:pb-8" {
          ul class="text-lg flex flex-wrap flex-row justify-center sm:items-center flex-grow" {
            (HeaderLink { href: "/", text: "Home" })
            (HeaderLink { href: "/posts", text: "Posts" })
            (HeaderLink { href: "/notes", text: "Notes" })
            (HeaderLink { href: "/videos", text: "Videos" })
            (HeaderLink { href: "/podcast", text: "Podcast" })
            (HeaderLink { href: "/projects", text: "Projects" })
            (HeaderLink { href: "/newsletter", text: "Newsletter" })
          }
        }
      }
    }
}
