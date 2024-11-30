use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};

use maud::{html, Markup};
use posts::{
    blog::{BlogPostPath, BlogPosts, MatchesPath, ToCanonicalPath},
    date::PostedOn,
    til::TilPosts,
    Post,
};
use rsky_lexicon::app::bsky::feed::{PostView, ThreadViewPost, ThreadViewPostEnum};
use rss::validation::Validate;
use tracing::instrument;

pub(crate) mod md;

use crate::{
    bsky::fetch_thread,
    http_server::{
        errors::ServerError,
        pages::blog::md::{
            html::MarkdownRenderContext, FindCoverPhoto, IntoHtml, SyntaxHighlightingContext,
        },
        templates::{base_constrained, header::OpenGraph, post_templates::BlogPostList, ShortDesc},
        LinkTo, ToRssItem,
    },
    AppConfig, AppState,
};

pub(crate) struct MyChannel(rss::Channel);

impl MyChannel {
    #[instrument(skip_all)]
    pub fn from_posts<T>(
        config: &AppConfig,
        context: &SyntaxHighlightingContext,
        posts: &[&Post<T>],
    ) -> cja::Result<Self>
    where
        Post<T>: ToRssItem,
    {
        let items: cja::Result<Vec<rss::Item>> = posts
            .iter()
            .map(|p| p.to_rss_item(config, context))
            .collect();

        Ok(Self::from_items(config, context, &items?))
    }

    pub fn from_items(
        config: &AppConfig,
        _context: &SyntaxHighlightingContext,
        items: &[rss::Item],
    ) -> Self {
        use rss::ChannelBuilder;

        let channel = ChannelBuilder::default()
            .title("coreyja Blog".to_string())
            .link(config.home_page())
            .copyright(Some("Copyright Corey Alexander".to_string()))
            .language(Some("en-us".to_string()))
            .items(items)
            .build();

        Self(channel)
    }

    pub fn validate(&self) -> Result<(), rss::validation::ValidationError> {
        self.0.validate()
    }
}

#[instrument(skip_all)]
pub(crate) async fn rss_feed(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
    let channel = MyChannel::from_posts(
        &state.app,
        &state.syntax_highlighting_context,
        &state.blog_posts.by_recency(),
    )?;

    Ok(channel.into_response())
}

#[instrument(skip_all)]
pub(crate) async fn full_rss_feed(
    State(state): State<AppState>,
    State(blog_posts): State<Arc<BlogPosts>>,
    State(til_posts): State<Arc<TilPosts>>,
) -> Result<impl IntoResponse, ServerError> {
    let mut items_with_date: Vec<(chrono::NaiveDate, rss::Item)> = vec![];
    for p in blog_posts.by_recency() {
        items_with_date.push((
            p.posted_on(),
            p.to_rss_item(&state.app, &state.syntax_highlighting_context)?,
        ));
    }
    for p in til_posts.by_recency() {
        items_with_date.push((
            p.posted_on(),
            p.to_rss_item(&state.app, &state.syntax_highlighting_context)?,
        ));
    }

    items_with_date.sort_by_key(|&(date, _)| std::cmp::Reverse(date));

    let items: Vec<rss::Item> = items_with_date.into_iter().map(|(_, i)| i).collect();

    let channel = MyChannel::from_items(&state.app, &state.syntax_highlighting_context, &items);

    Ok(channel.into_response())
}

impl IntoResponse for MyChannel {
    fn into_response(self) -> Response {
        let r = Response::builder()
            .header("Content-Type", "application/rss+xml")
            .body(self.0.to_string());

        if let Ok(r) = r {
            r.into_response()
        } else {
            let e: ServerError =
                cja::color_eyre::eyre::eyre!("Failed to build RSS Feed response").into();
            e.into_response()
        }
    }
}

#[instrument(skip_all)]
pub(crate) async fn posts_index(State(posts): State<Arc<BlogPosts>>) -> Result<Markup, StatusCode> {
    Ok(base_constrained(
        html! {
          h1 class="text-3xl" { "Blog Posts" }
          (BlogPostList(posts.by_recency()))
        },
        OpenGraph::default(),
    ))
}

#[instrument(skip(state, posts))]
pub(crate) async fn post_get(
    State(state): State<AppState>,
    State(posts): State<Arc<BlogPosts>>,
    Path(key): Path<String>,
) -> Result<Response, StatusCode> {
    {
        let path = BlogPostPath::new(key.clone());
        if path.file_exists() && !path.file_is_markdown() {
            return Ok(path.raw_bytes().into_response());
        }
    }

    let (post, m) = posts
        .posts()
        .iter()
        .find_map(|p| p.matches_path(&key).map(|m| (p, m)))
        .ok_or(StatusCode::NOT_FOUND)?;

    if let MatchesPath::RedirectToCanonicalPath = m {
        return Ok(
            Redirect::permanent(&format!("/posts/{}", post.path.canonical_path())).into_response(),
        );
    }

    let markdown = post.markdown();
    let cover_photo = markdown.ast.0.cover_photo();

    let context = MarkdownRenderContext {
        syntax_highlighting: state.syntax_highlighting_context.clone(),
        current_article_path: post.relative_link(),
    };

    let html = match markdown.ast.into_html(&state.app, &context) {
        Ok(html) => html,
        Err(e) => {
            let server_error = ServerError(e, StatusCode::INTERNAL_SERVER_ERROR);
            tracing::error!(?server_error, "Failed to render markdown to html");

            return Err(server_error.1);
        }
    };

    let image_defaulted_open_graph = match cover_photo {
        Some(cover_photo) => OpenGraph {
            image: Some(cover_photo),
            ..OpenGraph::default()
        },
        None => OpenGraph::default(),
    };

    let bsky_thread = if let Some(bsky_post_url) = &post.frontmatter.bsky_url {
        Some((bsky_post_url, fetch_thread(bsky_post_url).await.unwrap()))
    } else {
        None
    };

    Ok(base_constrained(
        html! {
          h1 class="text-2xl" { (post.markdown().title) }
          subtitle class="block text-lg text-subtitle mb-8" { (post.markdown().date) }

          div {
            (html)
          }

          @if let Some((url, thread)) = bsky_thread {
            div class="mt-8" {
              (bluesky_post_stats(&url, &thread))
              (bsky_comments(&url, thread))
            }
          }
        },
        OpenGraph {
            title: post.markdown().title,
            r#type: "article".to_string(),
            description: post.short_description(),
            ..image_defaulted_open_graph
        },
    )
    .into_response())
}

fn bluesky_post_stats(url: &str, thread: &ThreadViewPost) -> Markup {
    html! {
        a href=(url) target="_blank" {
            p class="flex items-center hover:underline gap-2 text-lg" {
                span class="flex items-center" {
                    svg color="pink" xmlns="http://www.w3.org/2000/svg" stroke="pink" fill="pink" stroke-width="1.5" class="size-5" viewBox="0 0 24 24" {
                        path stroke-linejoin="round" d="M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12Z" stroke-linecap="round" {}
                    }
                    span class="ml-1" {
                        (thread.post.like_count.unwrap_or(0)) " likes"
                    }
                }
                span class="flex items-center" {
                    svg viewBox="0 0 24 24" class="size-5" stroke-width="1.5" xmlns="http://www.w3.org/2000/svg" stroke="green" fill="none" {
                        path stroke-linecap="round" stroke-linejoin="round" d="M19.5 12c0-1.232-.046-2.453-.138-3.662a4.006 4.006 0 0 0-3.7-3.7 48.678 48.678 0 0 0-7.324 0 4.006 4.006 0 0 0-3.7 3.7c-.017.22-.032.441-.046.662M19.5 12l3-3m-3 3-3-3m-12 3c0 1.232.046 2.453.138 3.662a4.006 4.006 0 0 0 3.7 3.7 48.656 48.656 0 0 0 7.324 0 4.006 4.006 0 0 0 3.7-3.7c.017-.22.032-.441.046-.662M4.5 12l3 3m-3-3-3 3" {}
                    }
                    span class="ml-1" {
                        (thread.post.repost_count.unwrap_or(0)) " reposts"
                    }
                }
                span class="flex items-center" {
                    svg stroke-width="1.5" stroke="#7FBADC" fill="#7FBADC" class="size-5" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" {
                        path stroke-linecap="round" stroke-linejoin="round" d="M12 20.25c4.97 0 9-3.694 9-8.25s-4.03-8.25-9-8.25S3 7.444 3 12c0 2.104.859 4.023 2.273 5.48.432.447.74 1.04.586 1.641a4.483 4.483 0 0 1-.923 1.785A5.969 5.969 0 0 0 6 21c1.282 0 2.47-.402 3.445-1.087.81.22 1.668.337 2.555.337Z" {}
                    }
                    span class="ml-1" {
                        (thread.post.reply_count.unwrap_or(0)) " replies"
                    }
                }
            }
        }
    }
}

fn bsky_comments(post_url: &str, thread: ThreadViewPost) -> Markup {
    html! {
        h2 class="mt-6 text-xl font-bold" {
            "Comments"
        }
        p class="mt-2 text-sm" {
            "Reply on Bluesky "
            a href=(post_url) target="_blank" rel="noreferrer noopener" class="underline" {
                "here"
            }
            " to join the conversation."
        }
        @if let Some(replies) = thread.replies {
            hr class="mt-2";
            div class="mt-2 space-y-8" {
                @for reply in replies {
                    @if let ThreadViewPostEnum::ThreadViewPost(reply) = *reply {
                        (bsky_comment(reply))
                    }
                }
            }
        }
    }
}

fn bsky_comment(comment: ThreadViewPost) -> Markup {
    let avatar_class_name = "h-6 w-6 shrink-0 rounded-full bg-gray-300";
    let author = &comment.post.author;

    html! {
        div class="my-4 text-sm" {
            div class="flex max-w-xl flex-col gap-2" {
                a href=(format!("//bsky.app/profile/{}", author.did)) target="_blank" rel="noreferrer noopener" {
                    @if let Some(avatar) = &author.avatar {
                        img class=(avatar_class_name) src=(avatar) alt="avatar";
                    } @else {
                        div class=(avatar_class_name) {}
                    }
                    p class="line-clamp-1" {
                        @if let Some(display_name) = &author.display_name {
                            (display_name)
                        }
                        " "
                        span class="text-gray-500" {
                            "@" (author.handle)
                        }
                    }
                }
                a href=(format!("//bsky.app/profile/{}/post/{}", author.did, comment.post.uri.split('/').last().unwrap())) target="_blank" rel="noreferrer noopener" {
                    p {
                        (comment.post.record.get("text").unwrap_or(&serde_json::Value::String(String::new())).as_str().unwrap_or(""))
                    }
                    (bsky_comment_actions(&comment.post))
                }
            }
            @if let Some(replies) = comment.replies {
                @for reply in replies {
                    @if let ThreadViewPostEnum::ThreadViewPost(reply) = *reply {
                        div class="border-l-2 border-neutral-600 pl-2" {
                            (bsky_comment(reply))
                        }
                    }
                }
            }
        }
    }
}

fn bsky_comment_actions(post: &PostView) -> Markup {
    html! {
        div class="mt-2 flex w-full max-w-[150px] flex-row items-center justify-between opacity-60" {
            div class="flex flex-row items-center gap-1.5" {
                svg viewBox="0 0 24 24" stroke-width="1.5" xmlns="http://www.w3.org/2000/svg" fill="none" class="size-4" stroke="currentColor" {
                    path d="M12 20.25c4.97 0 9-3.694 9-8.25s-4.03-8.25-9-8.25S3 7.444 3 12c0 2.104.859 4.023 2.273 5.48.432.447.74 1.04.586 1.641a4.483 4.483 0 0 1-.923 1.785A5.969 5.969 0 0 0 6 21c1.282 0 2.47-.402 3.445-1.087.81.22 1.668.337 2.555.337Z" stroke-linejoin="round" stroke-linecap="round" {}
                }
                p class="text-xs" {
                    (post.reply_count.unwrap_or(0))
                }
            }
            div class="flex flex-row items-center gap-1.5" {
                svg viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" xmlns="http://www.w3.org/2000/svg" fill="none" class="size-4" {
                    path stroke-linecap="round" d="M19.5 12c0-1.232-.046-2.453-.138-3.662a4.006 4.006 0 0 0-3.7-3.7 48.678 48.678 0 0 0-7.324 0 4.006 4.006 0 0 0-3.7 3.7c-.017.22-.032.441-.046.662M19.5 12l3-3m-3 3-3-3m-12 3c0 1.232.046 2.453.138 3.662a4.006 4.006 0 0 0 3.7 3.7 48.656 48.656 0 0 0 7.324 0 4.006 4.006 0 0 0 3.7-3.7c.017-.22.032-.441.046-.662M4.5 12l3 3m-3-3-3 3" stroke-linejoin="round" {}
                }
                p class="text-xs" {
                    (post.repost_count.unwrap_or(0))
                }
            }
            div class="flex flex-row items-center gap-1.5" {
                svg viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-4" xmlns="http://www.w3.org/2000/svg" fill="none" {
                    path stroke-linejoin="round" d="M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12Z" stroke-linecap="round" {}
                }
                p class="text-xs" {
                    (post.like_count.unwrap_or(0))
                }
            }
        }
    }
}
