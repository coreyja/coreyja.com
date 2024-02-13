use crate::components::buttons::LinkButton;
use crate::components::header::Header;
use crate::error_template::{AppError, ErrorTemplate};
use crate::components::constrained_width::ConstrainedWidth;

use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/leptos_site.css"/>

        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
        <link href="https://fonts.googleapis.com/css2?family=Quicksand:wght@300;400;500;600;700&&display=swap" rel="stylesheet" />

        <meta name="theme-color" content="#AE93ED" />

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <main class="text-text font-sans min-h-screen flex flex-col" >
                <ConstrainedWidth>
                    <Header />
                    <Routes>
                        <Route path="" view=HomePage ssr=SsrMode::Async />
                    </Routes>
                </ConstrainedWidth>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
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
                    <LinkButton text="View Posts" href="/posts" primary=true />
                </div>
            </div>
        </div>
    }
}
