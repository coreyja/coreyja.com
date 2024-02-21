use std::rc::Rc;

use leptos::*;
use leptos_query::QueryResult;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Til {
    slug: String,
    href: String,
    title: String,
    date: String,
}

#[server]
async fn get_recent_tils(_id: ()) -> Result<Vec<Til>, ServerFnError> {
    impl From<&posts::til::TilPost> for Til {
        fn from(post: &posts::til::TilPost) -> Self {
            Self {
                slug: post.frontmatter.slug.clone(),
                href: format!("/til/{}", post.frontmatter.slug),
                title: post.frontmatter.title.clone(),
                date: post.frontmatter.date.to_string(),
            }
        }
    }

    let state = crate::server::extractors::extract_state()?;
    let mut recent_tils = state.til_posts.by_recency();
    recent_tils.truncate(3);

    let recent: Vec<_> = recent_tils.into_iter().map(|til| til.into()).collect();

    Ok(recent)
}

fn use_get_recent_tils(
) -> QueryResult<Result<Vec<Til>, ServerFnError>, impl leptos_query::RefetchFn> {
    leptos_query::use_query(|| (), get_recent_tils, Default::default())
}

#[component]
pub fn RecentTils() -> impl IntoView {
    let QueryResult { data, .. } = use_get_recent_tils();
    let tils = move || data.get().map(|tils| tils.clone().unwrap_or_default());

    view! {
        <Transition>
            {move || {
                tils()
                    .map(|tils| {
                        view! {
                            <ul>
                                <For
                                    each=move || tils.clone()
                                    key=move |data| data.slug.clone()
                                    let:til
                                >
                                    <li class="my-4">
                                        <a href=til.href>
                                            <span class="text-subtitle text-sm inline-block w-[80px]">
                                                {til.date.clone()}
                                            </span>
                                            " "
                                            {til.title}
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
