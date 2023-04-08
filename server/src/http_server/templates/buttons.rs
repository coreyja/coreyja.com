use maud::{html, Markup, Render};

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
