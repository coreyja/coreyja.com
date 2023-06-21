use std::sync::Arc;

use axum::extract::State;
use maud::{html, Markup};

use crate::{
    http_server::templates::{base, buttons::Button, posts::TilPostList},
    posts::til::TilPosts,
};

pub(crate) async fn home_page(State(til_posts): State<Arc<TilPosts>>) -> Markup {
    let mut recent_tils = til_posts.by_recency();
    recent_tils.truncate(3);

    base(html! {
        ."bg-header-background bg-cover bg-left bg-no-repeat" {
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
        }

        p."pt-16"  {
            "Hello! You stumbled upon the beta version for my personal site. To see the live version, go to "
            a href="https://coreyja.com" { "coreyja.com" }
        }

        p ."mb-8" {
            "Right now this is mostly powering a personal Discord bot. In the future it will be the home for everying `coreyja` branded!"
        }

        div {
            h2 ."text-3xl" { "Recent TILs" }
            (TilPostList(recent_tils))
        }
    })
}
