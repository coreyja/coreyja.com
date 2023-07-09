use maud::{html, Markup};

mod footer;

pub use footer::footer;

const LOGO_SVG: &str = include_str!("../../../static/logo.svg");
const LOGO_MONOCHROME_SVG: &str = include_str!("../../../static/logo-monochrome.svg");

const MAX_WIDTH_CONTAINER_CLASSES: &str = "max-w-5xl m-auto px-4";

mod header;
pub use header::{head, header};

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
        (constrained_width(header()))

        (inner)

        (footer())
      }
    }
}

pub fn base_constrained(inner: Markup) -> Markup {
    base(constrained_width(inner))
}

pub fn constrained_width(inner: Markup) -> Markup {
    html! {
      div ."w-full ".(MAX_WIDTH_CONTAINER_CLASSES) {
        (inner)
      }
    }
}

pub(crate) mod buttons;
pub(crate) mod posts;

pub(crate) mod newsletter;
