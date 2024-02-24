use leptos::*;

use crate::components::recent_posts::RecentPosts;

#[component]
pub fn PostsIndex() -> impl IntoView {
    view! {
        <h1 class="text-3xl">"Blog Posts"</h1>

        <RecentPosts limit=None/>

        <div class="mb-24"></div>
    }
}
