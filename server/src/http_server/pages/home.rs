use std::sync::Arc;

use axum::extract::State;
use maud::{html, Markup};
use miette::{Context, IntoDiagnostic};
use posts::{blog::BlogPosts, past_streams::PastStreams, til::TilPosts};

use crate::{
    http_server::{
        pages::videos::{VideoList, YoutubeVideo},
        templates::{
            base,
            buttons::LinkButton,
            constrained_width,
            post_templates::{BlogPostList, TilPostList},
        },
        MietteError,
    },
    AppState,
};

pub(crate) async fn home_page(
    State(app_state): State<AppState>,
    State(til_posts): State<Arc<TilPosts>>,
    State(blog_posts): State<Arc<BlogPosts>>,
    State(part_streams): State<Arc<PastStreams>>,
) -> Result<Markup, MietteError> {
    let mut recent_tils = til_posts.by_recency();
    recent_tils.truncate(3);

    let mut recent_posts = blog_posts.by_recency();
    recent_posts.truncate(3);

    let mut recent_streams = part_streams.by_recency();
    recent_streams.truncate(3);

    let recent_videos = sqlx::query_as!(
        YoutubeVideo,
        "SELECT *
        FROM YoutubeVideos
        ORDER BY published_at DESC LIMIT 3"
    )
    .fetch_all(&app_state.db)
    .await
    .into_diagnostic()?;

    sqlx::query!("SELECT * FROM USERS WHERE true = false",)
        .fetch_one(&app_state.db)
        .await
        .into_diagnostic()
        .context("Failed to fetch user")?;

    Ok(base(
        html! {
            (constrained_width(html! {
                ."md:bg-header-background bg-cover bg-left bg-no-repeat mb-24" {
                    ."md:w-[60%]" {
                        h1 class="text-2xl sm:text-4xl font-medium leading-tight pt-8 md:pt-16 pb-4" {
                            "Educational & entertaining content for developers of all skill levels"
                        }

                        h3 class="text-lg sm:text-2xl text-subtitle leading-tight mb-8" {
                            "My goal is to make you feel at home and help you grow your skills through my streams, videos and posts."
                        }

                        div class="text-xl flex flex-row space-x-8" {
                            (LinkButton::primary(html!("View Posts"), "/posts"))
                        }
                    }
                }

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
                            h2 ."text-3xl" { a href="/streams" { "Recent Videos" } }
                            (VideoList(recent_videos))
                        }
                    }
                }

            }))

        },
        Default::default(),
    ))
}
