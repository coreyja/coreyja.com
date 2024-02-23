use leptos::*;

#[component]
pub fn NewsletterSignup() -> impl IntoView {
    view! {
        <div class="bg-[rgba(178,132,255,0.1)] py-16 flex flex-col items-center space-y-8 px-4">
            <h2 class="text-3xl leading-none">coreyja weekly</h2>
            <p class="max-x-prose leading-loose">
                My weekly newsletter tailored at developers who are eager to grow with me! <br/>
                Every week will be unique, but expect topics focusing around Web Development and Rust
            </p>
            <form
                action="https://app.convertkit.com/forms/5312462/subscriptions"
                method="post"
                class="w-full max-w-md flex flex-row gap-4"
            >
                <input
                    type="email"
                    name="email_address"
                    class="flex-grow py-2 px-2 rounded-md text-grey-999"
                    placeholder="Enter your email address"
                    required="required"
                />
                <input
                    type="submit"
                    value="Subscribe"
                    class="bg-secondary-400 rounded-lg px-8 py-2"
                />
            </form>
        </div>
    }
}

#[component]
pub fn Footer() -> impl IntoView {
    use crate::components::logos::dark_on_light::*;

    view! {
        <footer class="min-h-[100px] bg-subtitle">
            <div class="flex  max-w-5xl m-auto px-4">
                <div class="max-w-[10rem] sm:max-w-[15rem] min-w-[100px] py-8 flex-grow">
                    <a href="/">
                        <MainLogo/>
                    </a>
                </div>
                <div class="flex-grow"></div>
                <ul class="flex flex-row items-center text-background space-x-4 sm:space-x-8 text-xl sm:text-2xl">
                    <a href="https://github.com/coreyja" target="_blank" rel="noopener noreferrer">
                        <i class="fa-brands fa-github"></i>
                    </a>
                    <a href="https://twitch.tv/coreyja" target="_blank" rel="noopener noreferrer">
                        <i class="fa-brands fa-twitch"></i>
                    </a>
                    <a
                        href="https://youtube.com/@coreyja"
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        <i class="fa-brands fa-youtube"></i>
                    </a>
                    <a href="https://toot.cat/@coreyja" target="_blank" rel="noopener noreferrer">
                        <i class="fa-brands fa-mastodon"></i>
                    </a>
                    <a href="/rss.xml" target="_blank" rel="noopener noreferrer">
                        <i class="fa-solid fa-rss"></i>
                    </a>
                </ul>
            </div>
        </footer>
    }
}
