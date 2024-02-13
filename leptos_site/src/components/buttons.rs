use leptos::*;
use leptos_router::*;

#[component]
pub fn LinkButton(href: &'static str, text: &'static str, primary: bool) -> impl IntoView {
  let primary_classes = if primary {
    "bg-secondary-400"
  } else {
    "bg-background border"
  };
  let classes = format!("text-text px-8 py-4 rounded font-semibold my-2 inline-block {}", primary_classes);

    view! {
        <A
          href=href
          class=classes
          >
            {text}
        </A>
    }
}
