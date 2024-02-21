use crate::components::constrained_width::ConstrainedWidth;
use crate::components::header::Header;
use crate::error_template::{AppError, ErrorTemplate};

use crate::pages::home_page::HomePage;

use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    leptos_query::provide_query_client();

    view! {
        <Stylesheet id="leptos" href="/pkg/leptos_site.css"/>

        <link rel="preconnect" href="https://fonts.googleapis.com"/>
        <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin/>
        <link
            href="https://fonts.googleapis.com/css2?family=Quicksand:wght@300;400;500;600;700&&display=swap"
            rel="stylesheet"
        />

        <meta name="theme-color" content="#AE93ED"/>

        <Title text="coreyja.com"/>

        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors/> }.into_view()
        }>
            <main class="text-text font-sans min-h-screen flex flex-col">
                <ConstrainedWidth>
                    <Header/>
                    <Routes>
                        <Route path="" view=HomePage/>
                    </Routes>
                </ConstrainedWidth>
            </main>
        </Router>
    }
}
