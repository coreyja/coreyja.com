use std::unreachable;

use markdown::mdast::*;
use maud::{html, Markup, PreEscaped};
use syntect::{
    highlighting::ThemeSet,
    html::{ClassStyle, ClassedHTMLGenerator},
    parsing::SyntaxSet,
};
use url::Url;

use crate::AppState;

#[derive(Debug, Clone)]
pub(crate) struct SyntaxHighlightingContext {
    pub(crate) theme: syntect::highlighting::Theme,
    pub(crate) syntax_set: syntect::parsing::SyntaxSet,
}

impl Default for SyntaxHighlightingContext {
    fn default() -> Self {
        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();

        SyntaxHighlightingContext {
            syntax_set: ps,
            theme: ts.themes.get("base16-ocean.dark").unwrap().clone(),
        }
    }
}
pub(crate) trait IntoHtml {
    fn into_html(self, state: &AppState) -> Markup;
}

impl IntoHtml for Root {
    fn into_html(self, state: &AppState) -> Markup {
        html! {
            (self.children.into_html(state))
        }
    }
}

impl IntoHtml for Node {
    fn into_html(self, state: &AppState) -> Markup {
        match self {
            Node::Root(r) => r.into_html(state),
            Node::BlockQuote(x) => x.into_html(state),
            Node::FootnoteDefinition(_) => html! {}, // Skipping for now
            Node::List(l) => l.into_html(state),
            Node::Yaml(y) => y.into_html(state),
            Node::Break(b) => b.into_html(state),
            Node::InlineCode(c) => c.into_html(state),
            Node::Delete(s) => s.into_html(state),
            Node::Emphasis(e) => e.into_html(state),
            Node::Html(h) => h.into_html(state),
            Node::Image(i) => i.into_html(state),
            Node::Link(l) => l.into_html(state),
            Node::Strong(s) => s.into_html(state),
            Node::Text(t) => t.into_html(state),
            Node::Code(c) => c.into_html(state),
            Node::Heading(h) => h.into_html(state),
            Node::Table(t) => t.into_html(state),
            Node::TableRow(r) => r.into_html(state),
            Node::TableCell(c) => c.into_html(state),
            Node::ListItem(i) => i.into_html(state),
            Node::Paragraph(p) => p.into_html(state),
            Node::ThematicBreak(b) => b.into_html(state),
            Node::MdxJsxFlowElement(_) => todo!(),
            Node::MdxjsEsm(_) => todo!(),
            Node::Toml(_) => todo!(),
            Node::InlineMath(_) => todo!(),
            Node::MdxTextExpression(_) => todo!(),
            Node::FootnoteReference(_) => todo!(),
            Node::ImageReference(_) => todo!(),
            Node::MdxJsxTextElement(_) => todo!(),
            Node::LinkReference(_) => todo!(),
            Node::Math(_) => todo!(),
            Node::MdxFlowExpression(_) => todo!(),
            Node::Definition(_) => todo!(),
        }
    }
}

impl IntoHtml for Html {
    fn into_html(self, _state: &AppState) -> Markup {
        html! { (PreEscaped(self.value)) }
    }
}

impl IntoHtml for Break {
    fn into_html(self, _state: &AppState) -> Markup {
        html! { br; }
    }
}

impl IntoHtml for Yaml {
    fn into_html(self, _state: &AppState) -> Markup {
        // We get Yaml in the Frontmatter, so we don't want to render it
        // to our HTML
        html! {}
    }
}

impl IntoHtml for Paragraph {
    fn into_html(self, state: &AppState) -> Markup {
        html! {
            p class="my-4 max-w-prose leading-loose" {
                (self.children.into_html(state))
            }
        }
    }
}

impl IntoHtml for ListItem {
    fn into_html(self, state: &AppState) -> Markup {
        html! {
            li {
                (self.children.into_html(state))
            }
        }
    }
}

impl IntoHtml for TableCell {
    fn into_html(self, state: &AppState) -> Markup {
        html! {
            td {
                (self.children.into_html(state))
            }
        }
    }
}

impl IntoHtml for TableRow {
    fn into_html(self, state: &AppState) -> Markup {
        html! {
            tr {
                (self.children.into_html(state))
            }
        }
    }
}

impl IntoHtml for Table {
    fn into_html(self, state: &AppState) -> Markup {
        html! {
            table {
                tbody {
                    (self.children.into_html(state))
                }
            }
        }
    }
}

impl IntoHtml for BlockQuote {
    fn into_html(self, state: &AppState) -> Markup {
        html! {
          blockquote {
            (self.children.into_html(state))
          }
        }
    }
}

impl IntoHtml for Text {
    fn into_html(self, _state: &AppState) -> Markup {
        html! {
            (self.value)
        }
    }
}

impl IntoHtml for Heading {
    fn into_html(self, state: &AppState) -> Markup {
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

        let content = self.children.into_html(state);
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
    fn into_html(self, state: &AppState) -> Markup {
        html! {
          @for node in self {
            (node.into_html(state))
          }
        }
    }
}

impl IntoHtml for List {
    fn into_html(self, state: &AppState) -> Markup {
        html! {
            @match self.ordered {
                true => { ol class="max-w-prose" { (self.children.into_html(state)) } },
                false => { ul class="max-w-prose" { (self.children.into_html(state)) } },
            }
        }
    }
}

impl IntoHtml for InlineCode {
    fn into_html(self, _state: &AppState) -> Markup {
        html! {
          code { (self.value) }
        }
    }
}

impl IntoHtml for Delete {
    fn into_html(self, state: &AppState) -> Markup {
        html! {
          del { (self.children.into_html(state)) }
        }
    }
}

impl IntoHtml for Emphasis {
    fn into_html(self, state: &AppState) -> Markup {
        html! {
          em { (self.children.into_html(state)) }
        }
    }
}

impl IntoHtml for Image {
    fn into_html(self, _state: &AppState) -> Markup {
        html! {
          img src=(self.url) alt=(self.alt) title=[self.title] class="px-8 my-8" loading="lazy" {}
        }
    }
}

impl IntoHtml for Link {
    fn into_html(self, state: &AppState) -> Markup {
        let parsed_base = Url::parse(&state.app.base_url);

        let replaced_url = if let Ok(parsed_base) = parsed_base {
            let parse_options = Url::options().base_url(Some(&parsed_base));
            let url = parse_options.parse(&self.url);

            if let Ok(url) = url {
                url.to_string()
            } else {
                self.url.to_string()
            }
        } else {
            self.url.to_string()
        };

        html! {
          a href=(replaced_url) title=[self.title] class="underline" { (self.children.into_html(state)) }
        }
    }
}

impl IntoHtml for Strong {
    fn into_html(self, state: &AppState) -> Markup {
        html! {
          strong { (self.children.into_html(state)) }
        }
    }
}

impl IntoHtml for Code {
    fn into_html(self, state: &AppState) -> Markup {
        use syntect::util::LinesWithEndings;

        let ps = &state.markdown_to_html_context.syntax_set;
        let syntax = self
            .lang
            .and_then(|lang| ps.find_syntax_by_token(&lang))
            .unwrap_or_else(|| ps.find_syntax_plain_text());

        let mut html_generator = ClassedHTMLGenerator::new_with_class_style(
            syntax,
            &state.markdown_to_html_context.syntax_set,
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

impl IntoHtml for ThematicBreak {
    fn into_html(self, _state: &AppState) -> Markup {
        html! {
          hr class="my-8 opacity-20";
        }
    }
}
