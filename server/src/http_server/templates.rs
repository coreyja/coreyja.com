use maud::{html, Markup, PreEscaped, Render};

const LOGO_SVG: &str = include_str!("../../static/logo.svg");

pub fn head() -> Markup {
    html! {
      head {
        title { "coreyja.com" }
        link rel="stylesheet" href="/styles/tailwind.css" {}

        link rel="preconnect" href="https://fonts.googleapis.com" {}
        link rel="preconnect" href="https://fonts.gstatic.com" crossorigin {}
        link href="https://fonts.googleapis.com/css2?family=Quicksand:wght@300;400;500;600;700&&display=swap" rel="stylesheet" {}
      }
    }
}

pub fn header() -> Markup {
    html! {
      div class="flex" {
        div class="max-w-lg min-w-[200px] py-4 flex-grow" {
          (PreEscaped(LOGO_SVG))
        }

        nav class="flex flex-grow justify-end w-full ml-16 max-w-[50%]" {
          ul class="flex flex-row items-center justify-between flex-grow" {
            li {
              a href="/" { "Home" }
            }

            li {
              a href="/posts" { "Posts" }
            }

            li {
              a href="/projects" { "Projects" }
            }

            li {
              a href="/streaming" { "Streaming" }
            }

            li {
              a href="/contact" { "Contact" }
            }
          }
        }
      }
    }
}

pub fn base(inner: Markup) -> Markup {
    html! {
      (head())

      body class="bg-background text-text px-4 max-w-5xl m-auto font-sans" {
        (header())

        (inner)
      }
    }
}

pub struct Button {
    inner: Markup,
    button_type: ButtonType,
    additional_classes: Option<String>,
}

impl Button {
    pub fn primary(inner: Markup) -> Self {
        Self {
            inner,
            button_type: ButtonType::Primary,
            additional_classes: None,
        }
    }

    pub fn secondary(inner: Markup) -> Self {
        Self {
            inner,
            button_type: ButtonType::Secondary,
            additional_classes: None,
        }
    }

    pub fn with_classes(mut self, classes: &str) -> Self {
        self.additional_classes = Some(classes.to_string());
        self
    }
}

pub enum ButtonType {
    Primary,
    Secondary,
}
impl ButtonType {
    fn classes(&self) -> &str {
        match &self {
            ButtonType::Primary => "bg-accent",
            ButtonType::Secondary => "bg-background border",
        }
    }
}

impl Render for Button {
    fn render(&self) -> Markup {
        let mut classes = vec![
            "text-text",
            "px-8",
            "py-2",
            "rounded",
            "font-semibold",
            "my-2",
            self.button_type.classes(),
        ];

        if let Some(additional_classes) = &self.additional_classes {
            classes.push(additional_classes);
        }
        let classes = classes.join(" ");

        html! {
          button class=(classes) {
            (self.inner)
          }
        }
    }
}
