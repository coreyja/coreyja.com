use maud::{html, Markup};

mod footer;

pub use footer::footer;

const LOGO_SVG: &str = include_str!("../../../static/logo.svg");
const LOGO_MONOCHROME_SVG: &str = include_str!("../../../static/logo-monochrome.svg");

const MAX_WIDTH_CONTAINER_CLASSES: &str = "max-w-5xl m-auto px-4";

pub mod header;
use ::posts::MarkdownAst;
pub use header::{head, header};
use posts::Post;

use crate::AppConfig;

use self::header::OpenGraph;

use super::pages::blog::md::{IntoHtml, IntoPlainText, SyntaxHighlightingContext};

pub fn base(inner: Markup, og: header::OpenGraph) -> Markup {
    html! {
      html prefix="og: https://ogp.me/ns#" {
        (head(og))

        body class="
        bg-background
        text-text
        font-sans
        min-h-screen
        flex
        flex-col
        " {
          (constrained_width(header()))

          (inner)

          (footer())
        }
      }
    }
}

pub fn base_constrained(inner: Markup, og: OpenGraph) -> Markup {
    base(constrained_width(inner), og)
}

pub fn constrained_width(inner: Markup) -> Markup {
    html! {
      div ."w-full ".(MAX_WIDTH_CONTAINER_CLASSES) {
        (inner)
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
        context: &SyntaxHighlightingContext,
    ) -> miette::Result<maud::Markup> {
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
