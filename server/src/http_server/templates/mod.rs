use std::borrow::Borrow;

use maud::{html, Markup};

mod footer;

pub use footer::footer;

const LOGO_FLAT_SVG: &str = include_str!("../../../static/logo-flat.svg");
const LOGO_DARK_SVG: &str = include_str!("../../../static/logo-dark.svg");
const LOGO_DARK_FLAT_SVG: &str = include_str!("../../../static/logo-dark-flat.svg");

const MAX_WIDTH_CONTAINER_CLASSES: &str = "max-w-5xl m-auto px-4";

pub mod header;
use ::posts::MarkdownAst;
pub use header::{head, header};
use posts::Post;

use color_eyre;


use crate::AppConfig;

use self::header::OpenGraph;

use crate::http_server::MarkdownRenderContext;

use super::pages::blog::md::{IntoHtml, IntoPlainText};

pub fn base(inner: impl Borrow<Markup>, og: header::OpenGraph) -> Markup {
    html! {
      html prefix="og: https://ogp.me/ns#" {
        (head(og))

        body class="
        bg-background
        text-text
        font-medium
        font-sans
        min-h-screen
        flex
        flex-col
        " {
          (constrained_width(header()))

          (inner.borrow())

          (footer())
        }
      }
    }
}

pub fn base_constrained(inner: Markup, og: OpenGraph) -> Markup {
    base(constrained_width(inner), og)
}

pub fn constrained_width(inner: impl std::borrow::Borrow<Markup>) -> Markup {
    html! {
      div ."w-full ".(MAX_WIDTH_CONTAINER_CLASSES) {
        (inner.borrow())
      }
    }
}

pub(crate) mod buttons;
pub(crate) mod post_templates;

pub(crate) mod newsletter;


impl IntoHtml for MarkdownAst {
    fn into_html(
        self,
        config: &AppConfig,
        context: &MarkdownRenderContext,
    ) -> color_eyre::Result<maud::Markup> {
        self.0.into_html(config, context)
    }
}

pub trait ShortDesc {
    fn short_description(&self) -> Option<String>;
}

impl<FrontMatter> ShortDesc for Post<FrontMatter> {
    fn short_description(&self) -> Option<String> {
        let contents = self.ast.0.plain_text();

        Some(contents.chars().take(100).collect())
    }
}
