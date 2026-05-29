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
    /// `(rel, href)` pairs emitted as `<link rel="..." href="..." />` in `<head>`.
    /// Used to point at `site.standard.document` / `site.standard.publication`
    /// records on the PDS for verification.
    pub head_links: Vec<(String, String)>,
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
            head_links: Vec::new(),
        }
    }
}

impl OpenGraph {
    /// Default `OpenGraph` with `url` populated to the site root from `config`. Use this
    /// for the home page or where the canonical URL truly is the site root.
    pub fn default_for(config: &AppConfig) -> Self {
        Self::default_for_path(config, "/")
    }

    /// Default `OpenGraph` with `url` populated to an absolute URL built from `config` and
    /// `path`. Use this for index/listing pages whose canonical URL is not the site root
    /// (e.g. `/posts`, `/podcast`, `/notes`).
    pub fn default_for_path(config: &AppConfig, path: &str) -> Self {
        Self {
            url: config.app_url(path),
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
          @for (rel, href) in &self.head_links {
            link rel=(rel) href=(href) {}
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

#[cfg(test)]
mod tests {
    use super::OpenGraph;
    use maud::Render;

    fn rendered(og: &OpenGraph) -> String {
        og.render().into_string()
    }

    #[test]
    fn default_does_not_emit_og_url() {
        // Regression: the old default shipped the bare string "coreyja.com" as og:url.
        // The new default leaves `url` empty, and Render must skip the tag entirely.
        let out = rendered(&OpenGraph::default());
        assert!(
            !out.contains(r#"property="og:url""#),
            "default OpenGraph should not emit og:url; got:\n{out}"
        );
    }

    #[test]
    fn populated_url_is_emitted() {
        let og = OpenGraph {
            url: "https://coreyja.com/posts/foo".to_string(),
            ..OpenGraph::default()
        };
        let out = rendered(&og);
        assert!(out.contains(r#"property="og:url""#));
        assert!(out.contains(r#"content="https://coreyja.com/posts/foo""#));
    }

    #[test]
    fn image_present_emits_default_twitter_card_and_mirrored_tags() {
        // With an image but no explicit twitter_card, Render should default to
        // summary_large_image and mirror twitter:title / twitter:description / twitter:image
        // from the og:* values.
        let og = OpenGraph {
            title: "Test Title".to_string(),
            description: Some("A description".to_string()),
            image: Some("https://example.com/img.png".to_string()),
            ..OpenGraph::default()
        };
        let out = rendered(&og);
        assert!(out.contains(r#"name="twitter:card" content="summary_large_image""#));
        assert!(out.contains(r#"name="twitter:title" content="Test Title""#));
        assert!(out.contains(r#"name="twitter:description" content="A description""#));
        assert!(out.contains(r#"name="twitter:image" content="https://example.com/img.png""#));
    }

    #[test]
    fn explicit_twitter_card_overrides_default() {
        let og = OpenGraph {
            image: Some("https://example.com/img.png".to_string()),
            twitter_card: Some("summary".to_string()),
            ..OpenGraph::default()
        };
        let out = rendered(&og);
        assert!(out.contains(r#"name="twitter:card" content="summary""#));
        assert!(!out.contains(r#"name="twitter:card" content="summary_large_image""#));
    }

    #[test]
    fn no_image_emits_no_twitter_tags() {
        let og = OpenGraph {
            image: None,
            description: Some("A description".to_string()),
            ..OpenGraph::default()
        };
        let out = rendered(&og);
        assert!(
            !out.contains(r#"name="twitter:card""#),
            "no image should suppress twitter:card; got:\n{out}"
        );
        assert!(!out.contains(r#"name="twitter:title""#));
        assert!(!out.contains(r#"name="twitter:description""#));
        assert!(!out.contains(r#"name="twitter:image""#));
    }

    #[test]
    fn twitter_description_omitted_when_description_missing() {
        let og = OpenGraph {
            image: Some("https://example.com/img.png".to_string()),
            description: None,
            ..OpenGraph::default()
        };
        let out = rendered(&og);
        // Image present → twitter:title and twitter:image still emit
        assert!(out.contains(r#"name="twitter:title""#));
        assert!(out.contains(r#"name="twitter:image""#));
        // but no twitter:description without an underlying og:description
        assert!(!out.contains(r#"name="twitter:description""#));
    }

    #[test]
    fn published_time_emitted_only_when_populated() {
        let with = OpenGraph {
            published_time: Some("2026-05-23T00:00:00+00:00".to_string()),
            ..OpenGraph::default()
        };
        let without = OpenGraph {
            published_time: None,
            ..OpenGraph::default()
        };
        assert!(rendered(&with).contains(r#"property="article:published_time""#));
        assert!(!rendered(&without).contains(r#"property="article:published_time""#));
    }

    #[test]
    fn multiple_tags_emit_one_article_tag_each() {
        let og = OpenGraph {
            tags: vec!["rust".to_string(), "axum".to_string()],
            ..OpenGraph::default()
        };
        let out = rendered(&og);
        let count = out.matches(r#"property="article:tag""#).count();
        assert_eq!(count, 2, "two tags should emit two article:tag elements");
        assert!(out.contains(r#"property="article:tag" content="rust""#));
        assert!(out.contains(r#"property="article:tag" content="axum""#));
    }

    #[test]
    fn empty_tags_emit_no_article_tag() {
        let og = OpenGraph::default();
        let out = rendered(&og);
        assert!(!out.contains(r#"property="article:tag""#));
    }

    #[test]
    fn head_links_emit_link_tags() {
        let og = OpenGraph {
            head_links: vec![
                (
                    "site.standard.document".to_string(),
                    "at://did:plc:abc/site.standard.document/post-1".to_string(),
                ),
                (
                    "site.standard.publication".to_string(),
                    "at://did:plc:abc/site.standard.publication/3xyz".to_string(),
                ),
            ],
            ..OpenGraph::default()
        };
        let out = rendered(&og);
        assert!(out.contains(r#"rel="site.standard.document""#));
        assert!(out.contains(r#"href="at://did:plc:abc/site.standard.document/post-1""#));
        assert!(out.contains(r#"rel="site.standard.publication""#));
        assert!(out.contains(r#"href="at://did:plc:abc/site.standard.publication/3xyz""#));
    }

    #[test]
    fn empty_head_links_emit_nothing() {
        let og = OpenGraph::default();
        let out = rendered(&og);
        assert!(!out.contains("site.standard.document"));
        assert!(!out.contains("site.standard.publication"));
    }

    #[test]
    fn new_optional_fields_emit_when_populated() {
        let og = OpenGraph {
            image: Some("https://example.com/img.png".to_string()),
            image_width: Some(1200),
            image_height: Some(630),
            image_alt: Some("alt text".to_string()),
            site_name: Some("coreyja".to_string()),
            locale: Some("en_US".to_string()),
            twitter_site: Some("@coreyja.com".to_string()),
            author: Some("Corey".to_string()),
            ..OpenGraph::default()
        };
        let out = rendered(&og);
        assert!(out.contains(r#"property="og:image:width" content="1200""#));
        assert!(out.contains(r#"property="og:image:height" content="630""#));
        assert!(out.contains(r#"property="og:image:alt" content="alt text""#));
        assert!(out.contains(r#"property="og:site_name" content="coreyja""#));
        assert!(out.contains(r#"property="og:locale" content="en_US""#));
        assert!(out.contains(r#"name="twitter:site" content="@coreyja.com""#));
        assert!(out.contains(r#"property="article:author" content="Corey""#));
    }
}
