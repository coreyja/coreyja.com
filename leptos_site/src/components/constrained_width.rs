use leptos::*;

#[component]
pub fn ConstrainedWidth(children: Children) -> impl IntoView {
    view! {
        <div class="w-full max-w-5xl mx-auto px-4">
            {children()}
        </div>
    }
}
