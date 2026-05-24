pub mod html;

pub(crate) use html::IntoHtml;
pub(crate) use html::SyntaxHighlightingContext;

mod plain;
pub(crate) use plain::IntoPlainText;

// `image` module is unused after the OG card rework moved cover-photo logic out of
// the OpenGraph chain. Trait kept in place; not currently re-exported.
#[allow(dead_code)]
mod image;
