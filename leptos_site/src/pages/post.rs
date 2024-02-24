use std::path::PathBuf;

use leptos::*;
use leptos_query::QueryResult;
use leptos_router::*;
use posts::blog::{BlogFrontMatter, BlogPost};
use syntect::parsing::SyntaxSet;

use crate::components::markdown::{Markdown, MarkdownContext};

struct RenderedBlogPost {
    pub frontmatter: BlogFrontMatter,
    pub rendered_html: String,
    pub path: PathBuf,
}

#[component]
pub fn Post(post: BlogPost, syntax_set: SyntaxSet) -> impl IntoView {
    let mut slug = post.path.clone();
    slug.pop();

    let context = MarkdownContext {
        asset_prefix: format!("posts/{}", slug.to_str().unwrap()),
        syntax_set,
    };

    view! {
        <h1 class="text-2xl">{post.title().to_string()}</h1>
        <span class="block text-lg text-subtitle mb-8">{format!("{}", post.frontmatter.date)}</span>

        <div>
            <Markdown ast=post.ast() context=context/>
        </div>
    }
}

#[server]
async fn get_post(slug: Option<String>) -> Result<Option<BlogPost>, ServerFnError> {
    let Some(slug) = slug else {
        return Ok(None);
    };
    let state = crate::server::extractors::extract_state()?;
    let post = state
        .blog_posts
        .posts()
        .iter()
        .find_map(|p| p.matches_path(&slug).map(|m| (p, m)));

    let Some(post) = post else {
        return Ok(None);
    };

    Ok(Some(post.0.clone()))
}

#[server]
async fn get_syntax_set(_arg: ()) -> Result<syntect::parsing::SyntaxSet, ServerFnError> {
    let state = crate::server::extractors::extract_state()?;

    Ok(state.markdown_to_html_context.syntax_set)
}

#[component]
pub fn PostPage() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.with(|p| p.get("slug").cloned());

    let QueryResult { data: post, .. } =
        leptos_query::use_query(slug, get_post, Default::default());

    let QueryResult {
        data: syntax_set, ..
    } = leptos_query::use_query(|| (), get_syntax_set, Default::default());

    view! {
        <Suspense>

            {move || {
                match (post(), syntax_set()) {
                    (Some(Ok(Some(post))), Some(Ok(syntax_set))) => {
                        view! { <Post post=post syntax_set=syntax_set/> }
                    }
                    (Some(Ok(None)), _) => view! { <div>"Post not found"</div> }.into_view(),
                    (_, None) => view! { <div>"Syntax Set not found"</div> }.into_view(),
                    (_, Some(Err(e))) => {
                        view! { <div>"Syntax Set errored" {format!("{}", e)}</div> }.into_view()
                    }
                    (Some(Err(e)), _) => {
                        view! { <div>"Error loading post:" {format!("{}", e)}</div> }.into_view()
                    }
                    (None, _) => view! { <div>"Loading"</div> }.into_view(),
                }
            }}

        </Suspense>
    }
}
