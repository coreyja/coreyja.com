use std::unreachable;

use markdown::mdast::*;
use maud::{html, Markup, PreEscaped};
use syntect::html::{ClassStyle, ClassedHTMLGenerator};

#[derive(Debug, Clone)]
pub(crate) struct HtmlRenderContext {
    pub(crate) theme: syntect::highlighting::Theme,
    pub(crate) syntax_set: syntect::parsing::SyntaxSet,
}
pub(crate) trait IntoHtml {
    fn into_html(self, context: &HtmlRenderContext) -> Markup;
}

impl IntoHtml for Root {
    fn into_html(self, context: &HtmlRenderContext) -> Markup {
        html! {
            (self.children.into_html(context))
        }
    }
}

impl IntoHtml for Node {
    fn into_html(self, context: &HtmlRenderContext) -> Markup {
        match self {
            Node::Root(r) => r.into_html(context),
            Node::BlockQuote(x) => x.into_html(context),
            Node::FootnoteDefinition(_) => html! {}, // Skipping for now
            Node::List(l) => l.into_html(context),
            Node::Yaml(y) => y.into_html(context),
            Node::Break(b) => b.into_html(context),
            Node::InlineCode(c) => c.into_html(context),
            Node::Delete(s) => s.into_html(context),
            Node::Emphasis(e) => e.into_html(context),
            Node::Html(h) => h.into_html(context),
            Node::Image(i) => i.into_html(context),
            Node::Link(l) => l.into_html(context),
            Node::Strong(s) => s.into_html(context),
            Node::Text(t) => t.into_html(context),
            Node::Code(c) => c.into_html(context),
            Node::Heading(h) => h.into_html(context),
            Node::Table(t) => t.into_html(context),
            Node::TableRow(r) => r.into_html(context),
            Node::TableCell(c) => c.into_html(context),
            Node::ListItem(i) => i.into_html(context),
            Node::Paragraph(p) => p.into_html(context),
            _ => todo!(),
        }
    }
}

impl IntoHtml for Html {
    fn into_html(self, _context: &HtmlRenderContext) -> Markup {
        html! { (PreEscaped(self.value)) }
    }
}

impl IntoHtml for Break {
    fn into_html(self, _context: &HtmlRenderContext) -> Markup {
        html! { br; }
    }
}

impl IntoHtml for Yaml {
    fn into_html(self, _context: &HtmlRenderContext) -> Markup {
        // We get Yaml in the Frontmatter, so we don't want to render it
        // to our HTML
        html! {}
    }
}

impl IntoHtml for Paragraph {
    fn into_html(self, context: &HtmlRenderContext) -> Markup {
        html! {
            p class="my-4 max-w-prose leading-loose" {
                (self.children.into_html(context))
            }
        }
    }
}

impl IntoHtml for ListItem {
    fn into_html(self, context: &HtmlRenderContext) -> Markup {
        html! {
            li {
                (self.children.into_html(context))
            }
        }
    }
}

impl IntoHtml for TableCell {
    fn into_html(self, context: &HtmlRenderContext) -> Markup {
        html! {
            td {
                (self.children.into_html(context))
            }
        }
    }
}

impl IntoHtml for TableRow {
    fn into_html(self, context: &HtmlRenderContext) -> Markup {
        html! {
            tr {
                (self.children.into_html(context))
            }
        }
    }
}

impl IntoHtml for Table {
    fn into_html(self, context: &HtmlRenderContext) -> Markup {
        html! {
            table {
                tbody {
                    (self.children.into_html(context))
                }
            }
        }
    }
}

impl IntoHtml for BlockQuote {
    fn into_html(self, context: &HtmlRenderContext) -> Markup {
        html! {
          blockquote {
            (self.children.into_html(context))
          }
        }
    }
}

impl IntoHtml for Text {
    fn into_html(self, _context: &HtmlRenderContext) -> Markup {
        html! {
            (self.value)
        }
    }
}

impl IntoHtml for Heading {
    fn into_html(self, context: &HtmlRenderContext) -> Markup {
        let id = self
            .children
            .iter()
            .map(|x| match x {
                Node::Text(t) => Ok(t.value.as_str()),
                _ => Err(miette::miette!("Heading should only contain text")),
            })
            .collect::<Result<String, _>>()
            .ok()
            .map(|x| x.to_lowercase().replace(' ', "-"));
        let href_attr = id.as_ref().map(|x| format!("#{}", x));

        let content = self.children.into_html(context);
        let inner = html! {
            @match self.depth {
                1 => h1 id=[id] class="max-w-prose text-2xl" { (content) },
                2 => h2 id=[id] class="max-w-prose text-xl" { (content) },
                3 => h3 id=[id] class="max-w-prose text-lg" { (content) },
                4 => h4 id=[id] class="max-w-prose text-lg text-subtitle" { (content) },
                5 => h5 id=[id] class="max-w-prose text-lg text-subtitle font-light" { (content) },
                6 => h6 id=[id] class="max-w-prose text-base text-subtitle" { (content) },
                #[allow(unreachable_code)]
                _ => (unreachable!("Invalid heading depth")),
            }
        };

        html! {
            @if let Some(href_attr) = href_attr {
                a href=(href_attr) {
                    (inner)
                }
            } @else {
                (inner)
            }
        }
    }
}

impl IntoHtml for Vec<Node> {
    fn into_html(self, context: &HtmlRenderContext) -> Markup {
        html! {
          @for node in self {
            (node.into_html(context))
          }
        }
    }
}

impl IntoHtml for List {
    fn into_html(self, context: &HtmlRenderContext) -> Markup {
        html! {
            @match self.ordered {
                true => { ol class="max-w-prose" { (self.children.into_html(context)) } },
                false => { ul class="max-w-prose" { (self.children.into_html(context)) } },
            }
        }
    }
}

impl IntoHtml for InlineCode {
    fn into_html(self, _context: &HtmlRenderContext) -> Markup {
        html! {
          code { (self.value) }
        }
    }
}

impl IntoHtml for Delete {
    fn into_html(self, context: &HtmlRenderContext) -> Markup {
        html! {
          del { (self.children.into_html(context)) }
        }
    }
}

impl IntoHtml for Emphasis {
    fn into_html(self, context: &HtmlRenderContext) -> Markup {
        html! {
          em { (self.children.into_html(context)) }
        }
    }
}

impl IntoHtml for Image {
    fn into_html(self, _context: &HtmlRenderContext) -> Markup {
        html! {
          img src=(self.url) alt=(self.alt) title=[self.title] class="px-8 my-8" {}
        }
    }
}

impl IntoHtml for Link {
    fn into_html(self, context: &HtmlRenderContext) -> Markup {
        html! {
          a href=(self.url) title=[self.title] class="underline" { (self.children.into_html(context)) }
        }
    }
}

impl IntoHtml for Strong {
    fn into_html(self, context: &HtmlRenderContext) -> Markup {
        html! {
          strong { (self.children.into_html(context)) }
        }
    }
}

impl IntoHtml for Code {
    fn into_html(self, context: &HtmlRenderContext) -> Markup {
        use syntect::util::LinesWithEndings;

        let ps = &context.syntax_set;
        let syntax = self
            .lang
            .and_then(|lang| ps.find_syntax_by_token(&lang))
            .unwrap_or_else(|| ps.find_syntax_plain_text());

        let mut html_generator = ClassedHTMLGenerator::new_with_class_style(
            syntax,
            &context.syntax_set,
            ClassStyle::Spaced,
        );

        for line in LinesWithEndings::from(&self.value) {
            html_generator
                .parse_html_for_line_which_includes_newline(line)
                .unwrap();
        }

        html! {
          pre class="my-4 py-4 bg-coding_background px-8 overflow-x-auto max-w-vw" { code { (PreEscaped(html_generator.finalize())) } }
        }
    }
}
