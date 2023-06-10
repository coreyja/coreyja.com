use maud::{html, Markup};
use reqwest::StatusCode;

use crate::http_server::templates::base;

pub(crate) async fn til_index() -> Result<Markup, StatusCode> {
    Ok(base(html! {
      h1 class="text-3xl" { "Today I Learned" }
    }))
}
