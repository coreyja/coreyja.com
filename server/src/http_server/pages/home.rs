use maud::{html, Markup};

use crate::http_server::templates::{base, Button};

pub async fn home_page() -> Markup {
    base(html! {
        ."w-[60%]" {
            h1 class="text-4xl font-medium leading-tight pt-16 pb-4" {
                "Creating Educational & Entertaining Content for Developers of All Skill Levels"
            }

            h3 class="text-2xl text-subtitle leading-tight pb-8" {
                "My goal is to make you feel at home and help you grow your skills through my streams and videos."
            }

            div class="text-xl" {
                (
                    Button::primary(html!("View Projects"))
                    .with_classes("mr-8")
                )

                (Button::secondary(html!("Learn about Corey")))
            }
        }

        p."pt-16"  {
            "Hello! You stumbled upon the beta version for my personal site. To see the live version, go to "
            a href="https://coreyja.com" { "coreyja.com" }
        }

        p {
            "Right now this is mostly powering a personal Discord bot. In the future it will be the home for everying `coreyja` branded!"
        }
    })
}
