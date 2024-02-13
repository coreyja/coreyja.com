use leptos::*;
use leptos_router::*;

#[component]
pub fn Header() -> impl IntoView {
    view! {
      <div class="flex flex-col justify-center items-stretch">
        <div class="flex flex-grow justify-center">
          <div class="max-w-sm min-w-[200px] py-8 lg:py-12 flex-grow">
            <A href="/" >
              <img src="/logo.svg" alt="Coreyja Logo" />
            </A>
          </div>
        </div>
      </div>

      <Nav />
    }
}

#[component]
fn Nav() -> impl IntoView {
    view! {
      <nav class="flex flex-grow w-full pb-4 sm:pb-8">
        <ul class="flex flex-col sm:flex-row justify-center sm:items-center flex-grow space-y-4 sm:space-y-0">
          <HeaderLink href="/" text="Home" />
          <HeaderLink href="/posts" text="Posts" />
          <HeaderLink href="/til" text="TILs" />
          <HeaderLink href="/videos" text="Videos" />
          <HeaderLink href="/projects" text="Projects" />
          <HeaderLink href="/newsletter" text="Newsletter" />
        </ul>
      </nav>
    }
}


#[component]
fn HeaderLink(href: &'static str, text: &'static str) -> impl IntoView {
    view! {
      <li class="sm:mx-8">
        <A href=href >
          {text}
        </A>
      </li>
    }
}
