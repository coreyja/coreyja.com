pub mod html;

pub(crate) use html::IntoHtml;
pub(crate) use html::SyntaxHighlightingContext;

mod plain;
pub(crate) use plain::IntoPlainText;

mod image;
pub(crate) use image::FindCoverPhoto;
