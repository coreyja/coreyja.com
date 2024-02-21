use std::rc::Rc;

use leptos::*;
use leptos_query::QueryResult;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Post {
    href: String,
    title: String,
    date: String,
}

#[server]
async fn get_recent_posts(_id: ()) -> Result<Vec<Post>, ServerFnError> {
    use posts::blog::ToCanonicalPath;

    impl From<&posts::blog::BlogPost> for Post {
        fn from(post: &posts::blog::BlogPost) -> Self {
            Self {
                href: format!("/posts/{}", post.path.canonical_path()),
                title: post.frontmatter.title.clone(),
                date: post.frontmatter.date.to_string(),
            }
        }
    }

    let state = crate::server::extractors::extract_state()?;
    let mut recent = state.blog_posts.by_recency();
    recent.truncate(3);

    let recent: Vec<_> = recent.into_iter().map(|til| til.into()).collect();

    Ok(recent)
}

fn use_get_recent_posts(
) -> QueryResult<Result<Vec<Post>, ServerFnError>, impl leptos_query::RefetchFn> {
    leptos_query::use_query(|| (), get_recent_posts, Default::default())
}

#[component]
pub fn RecentPosts() -> impl IntoView {
    let QueryResult { data, .. } = use_get_recent_posts();
    let posts = move || data.get().map(|posts| posts.clone().unwrap_or_default());

    view! {
        <Transition>
            {move || {
                posts()
                    .map(|posts| {
                        view! {
                            <ul>
                                <For
                                    each=move || posts.clone()
                                    key=move |data| data.title.clone()
                                    let:post
                                >
                                    <li class="my-4">
                                        <a href=post.href>
                                            <span class="text-subtitle text-sm inline-block w-[80px]">
                                                {post.date.clone()}
                                            </span>
                                            " "
                                            {post.title}
                                        </a>
                                    </li>
                                </For>
                            </ul>
                        }
                    })
            }}

        </Transition>
    }
}
