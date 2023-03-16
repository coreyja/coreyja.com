use maud::{html, Markup};

pub fn head() -> Markup {
    html! {
      head {
        title { "coreyja.com" }
        link rel="stylesheet" href="/styles/tailwind.css" {}
      }
    }
}

pub fn base(inner: Markup) -> Markup {
    html! {
      (head())

      body class="bg-background text-white" {
        (inner)
      }
    }
}
