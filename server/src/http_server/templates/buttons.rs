use maud::{html, Markup, Render};

pub struct LinkButton {
    inner: Markup,
    href: String,
    button_type: ButtonType,
    additional_classes: Option<String>,
}

impl LinkButton {
    pub fn primary(inner: Markup, href: impl Into<String>) -> Self {
        Self {
            inner,
            href: href.into(),
            button_type: ButtonType::Primary,
            additional_classes: None,
        }
    }

    pub fn secondary(inner: Markup, href: impl Into<String>) -> Self {
        Self {
            inner,
            href: href.into(),
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
            ButtonType::Primary => "bg-berryBlue text-almostBackground",
            ButtonType::Secondary => "bg-background border",
        }
    }
}

impl Render for LinkButton {
    fn render(&self) -> Markup {
        let mut classes = vec![
            "px-8",
            "py-4",
            "rounded",
            "font-semibold",
            "my-2",
            "inline-block",
            self.button_type.classes(),
        ];

        if let Some(additional_classes) = &self.additional_classes {
            classes.push(additional_classes);
        }
        let classes = classes.join(" ");

        html! {
          a href=(self.href) class=(classes) {
            (self.inner)
          }
        }
    }
}
