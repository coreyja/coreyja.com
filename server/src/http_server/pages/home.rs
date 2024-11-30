use std::sync::Arc;

use axum::extract::State;
use maud::{html, Markup};
use posts::{blog::BlogPosts, til::TilPosts};

use crate::{
    http_server::{
        pages::videos::{VideoList, YoutubeVideo},
        templates::{
            base,
            buttons::LinkButton,
            constrained_width,
            header::OpenGraph,
            post_templates::{BlogPostList, TilPostList},
        },
        ServerError,
    },
    AppState,
};

pub(crate) async fn home_page(
    State(app_state): State<AppState>,
    State(til_posts): State<Arc<TilPosts>>,
    State(blog_posts): State<Arc<BlogPosts>>,
) -> Result<Markup, ServerError> {
    let mut recent_tils = til_posts.by_recency();
    recent_tils.truncate(3);

    let mut recent_posts = blog_posts.by_recency();
    recent_posts.truncate(3);

    let recent_videos = sqlx::query_as!(
        YoutubeVideo,
        "SELECT *
        FROM YoutubeVideos
        ORDER BY published_at DESC LIMIT 3"
    )
    .fetch_all(&app_state.db)
    .await?;

    // let bytes = get_most_recent_bytes();
    // let most_recent_byte = bytes.first();

    // let top_3_bytes: Vec<Byte> = bytes.iter().take(3).cloned().collect();

    Ok(base(
        html! {
            (constrained_width(html! {
                ."flex bg-right-bottom bg-no-repeat mb-24 justify-between" {
                    ."md:w-[60%]" {
                        h1 class="text-2xl sm:text-4xl font-medium leading-tight pt-8 md:pt-16 pb-4" {
                            "Learn, Code, Develop"
                        }

                        h3 class="text-lg sm:text-2xl text-subtitle leading-tight mb-8" {
                            "My goal is to make you feel at home and help you grow your skills through my streams, videos and posts."
                        }

                        div class="text-xl flex flex-row space-x-8" {
                            (LinkButton::primary(html!("View Posts"), "/posts"))
                            // @if let Some(most_recent_byte) = most_recent_byte {
                            //     (LinkButton::primary(html!("Play Latest Byte"), most_recent_byte.relative_link()))
                            // }
                        }
                    }

                    div class="hidden md:w-[35%] md:flex" {
                        img src="/static/corey_8x_flipped.png" alt="Corey's Pixel Avatar Headshot" class="mt-auto" {}
                    }
                }

                // div class="mb-8" {
                //     h2 ."text-3xl" { a href="/bytes" { "Recent Bytes" } }
                //     h3 class="text-xl mb-4 text-gray-500" { "Code Review Challenges" }

                //     (ByteList::new(top_3_bytes))

                //     (LinkButton::primary(html!("View All Bytes"), "/bytes"))
                // }

                div class="flex flex-col md:flex-row md:space-x-8" {
                    div class="flex-grow" {
                        div ."mb-16" {
                            h2 ."text-3xl" { a href="/til" { "Recent TILs" } }
                            (TilPostList(recent_tils))
                        }

                        div ."mb-16" {
                            h2 ."text-3xl" { a href="/posts" { "Recent Blog Posts" } }
                            (BlogPostList(recent_posts))
                        }
                    }

                    div class="w-[320px]" {
                        div ."mb-16" {
                            h2 ."text-3xl" { a href="/videos" { "Recent Videos" } }
                            (VideoList(recent_videos))
                        }
                    }
                }

            }))

        },
        OpenGraph::default(),
    ))
}
