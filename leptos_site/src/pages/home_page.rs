use leptos::*;

use crate::components::{
    buttons::LinkButton, recent_posts::RecentPosts, recent_tils::RecentTils,
    recent_videos::RecentVideos,
};

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="md:bg-header-background bg-cover bg-left bg-no-repeat mb-24">
            <div class="md:w-[60%]">
                <h1 class="text-2xl sm:text-4xl font-medium leading-tight pt-8 md:pt-16 pb-4">
                    "Educational & entertaining content for developers of all skill levels"
                </h1>

                <h3 class="text-lg sm:text-2xl text-subtitle leading-tight mb-8">
                    "My goal is to make you feel at home and help you grow your skills through my streams, videos and posts."
                </h3>

                <div class="text-xl flex flex-row space-x-8">
                    <LinkButton text="View Posts" href="/posts" primary=true/>
                </div>
            </div>
        </div>

        <div class="flex flex-col md:flex-row md:space-x-8">
            <div class="flex-grow">
                <div class="mb-16">
                    <h2 class="text-3xl">
                        <a href="/til">"Recent TILs"</a>
                    </h2>
                    <RecentTils/>
                </div>

                <div class="mb-16">
                    <h2 class="text-3xl">
                        <a href="/posts">"Recent Blog Posts"</a>
                    </h2>
                    <RecentPosts limit=Some(3)/>
                </div>
            </div>

            <div class="w-[320px]">
                <h2 class="text-3xl">
                    <a href="/videos">"Recent Videos"</a>
                </h2>
                <RecentVideos/>
            </div>
        </div>
    }
}
