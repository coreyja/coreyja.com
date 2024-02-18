use leptos::*;

use crate::components::buttons::LinkButton;

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
    }
}
