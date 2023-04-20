use markdown::mdast::*;
use maud::{html, Markup, PreEscaped};
use syntect::html::{css_for_theme_with_class_style, ClassStyle, ClassedHTMLGenerator};

pub(crate) trait IntoHtml {
    fn into_html(self) -> Markup;
}

impl IntoHtml for Root {
    fn into_html(self) -> Markup {
        html! {
            (self.children.into_html())
        }
    }
}

impl IntoHtml for Node {
    fn into_html(self) -> Markup {
        match self {
            Node::Root(r) => r.into_html(),
            Node::BlockQuote(x) => x.into_html(),
            Node::FootnoteDefinition(_) => html! {}, // Skipping for now
            Node::List(l) => l.into_html(),
            Node::Yaml(y) => y.into_html(),
            Node::Break(b) => b.into_html(),
            Node::InlineCode(c) => c.into_html(),
            Node::Delete(s) => s.into_html(),
            Node::Emphasis(e) => e.into_html(),
            Node::Html(h) => h.into_html(),
            Node::Image(i) => i.into_html(),
            Node::Link(l) => l.into_html(),
            Node::Strong(s) => s.into_html(),
            Node::Text(t) => t.into_html(),
            Node::Code(c) => c.into_html(),
            Node::Heading(h) => h.into_html(),
            Node::Table(t) => t.into_html(),
            Node::TableRow(r) => r.into_html(),
            Node::TableCell(c) => c.into_html(),
            Node::ListItem(i) => i.into_html(),
            Node::Paragraph(p) => p.into_html(),
            _ => todo!(),
        }
    }
}

impl IntoHtml for Html {
    fn into_html(self) -> Markup {
        html! { (PreEscaped(self.value)) }
    }
}

impl IntoHtml for Break {
    fn into_html(self) -> Markup {
        html! { br; }
    }
}

impl IntoHtml for Yaml {
    fn into_html(self) -> Markup {
        // We get Yaml in the Frontmatter, so we don't want to render it
        // to our HTML
        html! {}
    }
}

impl IntoHtml for Paragraph {
    fn into_html(self) -> Markup {
        html! {
            p {
                (self.children.into_html())
            }
        }
    }
}

impl IntoHtml for ListItem {
    fn into_html(self) -> Markup {
        html! {
            li {
                (self.children.into_html())
            }
        }
    }
}

impl IntoHtml for TableCell {
    fn into_html(self) -> Markup {
        html! {
            td {
                (self.children.into_html())
            }
        }
    }
}

impl IntoHtml for TableRow {
    fn into_html(self) -> Markup {
        html! {
            tr {
                (self.children.into_html())
            }
        }
    }
}

impl IntoHtml for Table {
    fn into_html(self) -> Markup {
        html! {
            table {
                tbody {
                    (self.children.into_html())
                }
            }
        }
    }
}

impl IntoHtml for BlockQuote {
    fn into_html(self) -> Markup {
        html! {
          blockquote {
            (self.children.into_html())
          }
        }
    }
}

impl IntoHtml for Text {
    fn into_html(self) -> Markup {
        html! {
          (self.value)
        }
    }
}

impl IntoHtml for Heading {
    fn into_html(self) -> Markup {
        let tag = match self.depth {
            1 => "h1",
            2 => "h2",
            3 => "h3",
            4 => "h4",
            5 => "h5",
            6 => "h6",
            _ => unreachable!(),
        };
        html! {
            (tag) { (self.children.into_html()) }
        }
    }
}

impl IntoHtml for Vec<Node> {
    fn into_html(self) -> Markup {
        html! {
          @for node in self {
            (node.into_html())
          }
        }
    }
}

impl IntoHtml for List {
    fn into_html(self) -> Markup {
        let tag = match self.ordered {
            true => "ol",
            false => "ul",
        };
        html! {
            (tag) {
                (self.children.into_html())
            }
        }
    }
}

impl IntoHtml for InlineCode {
    fn into_html(self) -> Markup {
        html! {
          code { (self.value) }
        }
    }
}

impl IntoHtml for Delete {
    fn into_html(self) -> Markup {
        html! {
          del { (self.children.into_html()) }
        }
    }
}

impl IntoHtml for Emphasis {
    fn into_html(self) -> Markup {
        html! {
          em { (self.children.into_html()) }
        }
    }
}

impl IntoHtml for Image {
    fn into_html(self) -> Markup {
        html! {
          img src=(self.url) alt=(self.alt) title=[self.title] {}
        }
    }
}

impl IntoHtml for Link {
    fn into_html(self) -> Markup {
        html! {
          a href=(self.url) title=[self.title] { (self.children.into_html()) }
        }
    }
}

impl IntoHtml for Strong {
    fn into_html(self) -> Markup {
        html! {
          strong { (self.children.into_html()) }
        }
    }
}

impl IntoHtml for Code {
    fn into_html(self) -> Markup {
        use syntect::highlighting::ThemeSet;
        use syntect::parsing::SyntaxSet;
        use syntect::util::LinesWithEndings;

        // Load these once at the start of your program
        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();

        let syntax = self
            .lang
            .and_then(|lang| ps.find_syntax_by_token(&lang))
            .unwrap_or_else(|| ps.find_syntax_plain_text());

        // let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
        let mut html_generator =
            ClassedHTMLGenerator::new_with_class_style(syntax, &ps, ClassStyle::Spaced);

        for line in LinesWithEndings::from(&self.value) {
            html_generator
                .parse_html_for_line_which_includes_newline(line)
                .unwrap();
        }
        let css =
            css_for_theme_with_class_style(&ts.themes["base16-ocean.dark"], ClassStyle::Spaced)
                .unwrap();
        html! {
          style { (PreEscaped(css)) }
          pre { code { (PreEscaped(html_generator.finalize())) } }
        }
    }
}
