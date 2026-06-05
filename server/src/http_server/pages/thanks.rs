use maud::{html, Markup};

use crate::http_server::templates::{base_constrained, header::OpenGraph};

pub(crate) async fn thanks() -> Markup {
    base_constrained(
        html! {
            h1 class="my-4 text-2xl" { "Thanks & Credits" }

            p class="my-4" {
                "This site wouldn't be possible without the help of some awesome people!"
            }

            h2 class="my-2 text-xl" { "Profile Picture" }
            p class="my-2" {
                "Photo by "
                a class="underline" href="https://misnina.com/" target="_blank" { "Nina" }
                "."
            }

            h2 class="my-2 text-xl" { "Design and Logo" }
            p class="my-2" {
                "Design and logo by Brandi."
            }
        },
        OpenGraph {
            title: "Thanks & Credits - coreyja.com".to_owned(),
            description: Some("Credits for those who helped with coreyja.com".to_owned()),
            url: "https://coreyja.com/thanks".to_owned(),
            ..OpenGraph::default()
        },
    )
}
