use std::path::PathBuf;
use std::unreachable;

use markdown::mdast::{
    BlockQuote, Break, Code, Delete, Emphasis, Heading, Html, Image, InlineCode, Link, List,
    ListItem, Node, Paragraph, Root, Strong, Table, TableCell, TableRow, Text, ThematicBreak, Yaml,
};
use maud::{html, Markup, PreEscaped};
use syntect::{
    highlighting::ThemeSet,
    html::{ClassStyle, ClassedHTMLGenerator},
    parsing::SyntaxSet,
};
use tracing::info;
use url::Url;

use color_eyre::Result;

use crate::AppConfig;

#[derive(Debug, Clone)]
pub struct SyntaxHighlightingContext {
    pub(crate) theme: syntect::highlighting::Theme,
    pub(crate) syntax_set: syntect::parsing::SyntaxSet,
    pub(crate) current_article_path: Option<String>,
}

impl Default for SyntaxHighlightingContext {
    fn default() -> Self {
        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();

        SyntaxHighlightingContext {
            syntax_set: ps,
            theme: ts
                .themes
                .get("base16-ocean.dark")
                .expect("This theme exists in the defaults")
                .clone(),
            current_article_path: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MarkdownRenderContext {
    pub syntax_highlighting: SyntaxHighlightingContext,
    pub current_article_path: String,
}

pub(crate) trait IntoHtml {
    fn into_html(self, config: &AppConfig, context: &MarkdownRenderContext) -> Result<Markup>;
}

impl IntoHtml for Root {
    fn into_html(self, config: &AppConfig, context: &MarkdownRenderContext) -> Result<Markup> {
        Ok(html! {
            (self.children.into_html(config, context)?)
        })
    }
}

impl IntoHtml for Node {
    fn into_html(self, config: &AppConfig, context: &MarkdownRenderContext) -> Result<Markup> {
        match self {
            Node::Root(r) => r.into_html(config, context),
            Node::BlockQuote(x) => x.into_html(config, context),
            Node::FootnoteDefinition(_) => Ok(html! {}), // Skipping for now
            Node::List(l) => l.into_html(config, context),
            Node::Yaml(y) => y.into_html(config, context),
            Node::Break(b) => b.into_html(config, context),
            Node::InlineCode(c) => c.into_html(config, context),
            Node::Delete(s) => s.into_html(config, context),
            Node::Emphasis(e) => e.into_html(config, context),
            Node::Html(h) => h.into_html(config, context),
            Node::Image(i) => i.into_html(config, context),
            Node::Link(l) => l.into_html(config, context),
            Node::Strong(s) => s.into_html(config, context),
            Node::Text(t) => t.into_html(config, context),
            Node::Code(c) => c.into_html(config, context),
            Node::Heading(h) => h.into_html(config, context),
            Node::Table(t) => t.into_html(config, context),
            Node::TableRow(r) => r.into_html(config, context),
            Node::TableCell(c) => c.into_html(config, context),
            Node::ListItem(i) => i.into_html(config, context),
            Node::Paragraph(p) => p.into_html(config, context),
            Node::ThematicBreak(b) => b.into_html(config, context),
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
    fn into_html(self, _config: &AppConfig, _context: &MarkdownRenderContext) -> Result<Markup> {
        Ok(html! { (PreEscaped(self.value)) })
    }
}

impl IntoHtml for Break {
    fn into_html(self, _config: &AppConfig, _context: &MarkdownRenderContext) -> Result<Markup> {
        Ok(html! { br; })
    }
}

impl IntoHtml for Yaml {
    fn into_html(self, _config: &AppConfig, _context: &MarkdownRenderContext) -> Result<Markup> {
        // We get Yaml in the Frontmatter, so we don't want to render it
        // to our HTML
        Ok(html! {})
    }
}

impl IntoHtml for Paragraph {
    fn into_html(self, config: &AppConfig, context: &MarkdownRenderContext) -> Result<Markup> {
        Ok(html! {
            p class="my-4 max-w-prose leading-loose" {
                (self.children.into_html(config, context)?)
            }
        })
    }
}

impl IntoHtml for ListItem {
    fn into_html(self, config: &AppConfig, context: &MarkdownRenderContext) -> Result<Markup> {
        Ok(html! {
            li {
                (self.children.into_html(config, context)?)
            }
        })
    }
}

impl IntoHtml for TableCell {
    fn into_html(self, config: &AppConfig, context: &MarkdownRenderContext) -> Result<Markup> {
        Ok(html! {
            td {
                (self.children.into_html(config, context)?)
            }
        })
    }
}

impl IntoHtml for TableRow {
    fn into_html(self, config: &AppConfig, context: &MarkdownRenderContext) -> Result<Markup> {
        Ok(html! {
            tr {
                (self.children.into_html(config, context)?)
            }
        })
    }
}

impl IntoHtml for Table {
    fn into_html(self, config: &AppConfig, context: &MarkdownRenderContext) -> Result<Markup> {
        Ok(html! {
            table {
                tbody {
                    (self.children.into_html(config, context)?)
                }
            }
        })
    }
}

impl IntoHtml for BlockQuote {
    fn into_html(self, config: &AppConfig, context: &MarkdownRenderContext) -> Result<Markup> {
        Ok(html! {
          blockquote {
            (self.children.into_html(config, context)?)
          }
        })
    }
}

impl IntoHtml for Text {
    fn into_html(
        self,
        _config: &AppConfig,
        _context: &MarkdownRenderContext,
    ) -> Result<Markup> {
        Ok(html! {
            (self.value)
        })
    }
}

impl IntoHtml for Heading {
    fn into_html(self, config: &AppConfig, context: &MarkdownRenderContext) -> Result<Markup> {
        let id = self
            .children
            .iter()
            .map(|x| match x {
                Node::Text(t) => Ok(t.value.as_str()),
                _ => Err(color_eyre::eyre::eyre!("Heading should only contain text")),
            })
            .collect::<Result<String, _>>()
            .ok()
            .map(|x| x.to_lowercase().replace(' ', "-"));
        let href_attr = id.as_ref().map(|x| format!("#{x}"));

        let child_content = self.children.into_html(config, context)?;
        let inner = html! {
            @match self.depth {
                1 => h1 id=[id] class="max-w-prose text-2xl" { (child_content) },
                2 => h2 id=[id] class="max-w-prose text-xl" { (child_content) },
                3 => h3 id=[id] class="max-w-prose text-lg" { (child_content) },
                4 => h4 id=[id] class="max-w-prose text-lg text-subtitle" { (child_content) },
                5 => h5 id=[id] class="max-w-prose text-lg text-subtitle font-light" { (child_content) },
                6 => h6 id=[id] class="max-w-prose text-base text-subtitle" { (child_content) },
                #[allow(unreachable_code)]
                _ => (unreachable!("Invalid heading depth")),
            }
        };

        Ok(html! {
            @if let Some(href_attr) = href_attr {
                a href=(href_attr) {
                    (inner)
                }
            } @else {
                (inner)
            }
        })
    }
}

impl IntoHtml for Vec<Node> {
    fn into_html(self, config: &AppConfig, context: &MarkdownRenderContext) -> Result<Markup> {
        Ok(html! {
          @for node in self {
            (node.into_html(config, context)?)
          }
        })
    }
}

impl IntoHtml for List {
    fn into_html(self, config: &AppConfig, context: &MarkdownRenderContext) -> Result<Markup> {
        Ok(html! {
            @let inner = self.children.into_html(config, context)?;
            @if self.ordered {
                ol class="max-w-prose" { (inner) }
            } else {
                ul class="max-w-prose" { (inner) }
            }
        })
    }
}

impl IntoHtml for InlineCode {
    fn into_html(
        self,
        _config: &AppConfig,
        _context: &MarkdownRenderContext,
    ) -> Result<Markup> {
        Ok(html! {
          code { (self.value) }
        })
    }
}

impl IntoHtml for Delete {
    fn into_html(self, config: &AppConfig, context: &MarkdownRenderContext) -> Result<Markup> {
        Ok(html! {
          del { (self.children.into_html(config, context)?) }
        })
    }
}

impl IntoHtml for Emphasis {
    fn into_html(self, config: &AppConfig, context: &MarkdownRenderContext) -> Result<Markup> {
        Ok(html! {
          em { (self.children.into_html(config, context)?) }
        })
    }
}

impl IntoHtml for Image {
    fn into_html(self, config: &AppConfig, context: &MarkdownRenderContext) -> Result<Markup> {
        let relative_url = PathBuf::new()
            .join(&context.current_article_path)
            .join(&self.url)
            .to_string_lossy()
            .to_string();

        let src = config.app_url(&relative_url);

        Ok(html! {
            img src=(&src) alt=(self.alt) title=[self.title] class="px-8 my-8" loading="lazy" {}
        })
    }
}

impl IntoHtml for Link {
    fn into_html(self, config: &AppConfig, context: &MarkdownRenderContext) -> Result<Markup> {
        let parsed_base = config.base_url.clone();

        let parse_options = Url::options().base_url(Some(&parsed_base));
        let url = parse_options.parse(&self.url);

        let replaced_url = url.map_or_else(|_| self.url.clone(), |url| url.to_string());

        Ok(html! {
          a href=(replaced_url) title=[self.title] class="underline" { (self.children.into_html(config, context)?) }
        })
    }
}

impl IntoHtml for Strong {
    fn into_html(self, config: &AppConfig, context: &MarkdownRenderContext) -> Result<Markup> {
        Ok(html! {
          strong { (self.children.into_html(config, context)?) }
        })
    }
}

impl IntoHtml for Code {
    fn into_html(self, _config: &AppConfig, context: &MarkdownRenderContext) -> Result<Markup> {
        use syntect::util::LinesWithEndings;

        let ps = &context.syntax_highlighting.syntax_set;
        let syntax = self
            .lang
            .and_then(|lang| ps.find_syntax_by_token(&lang))
            .unwrap_or_else(|| ps.find_syntax_plain_text());

        let mut html_generator = ClassedHTMLGenerator::new_with_class_style(
            syntax,
            &context.syntax_highlighting.syntax_set,
            ClassStyle::Spaced,
        );

        for line in LinesWithEndings::from(&self.value) {
            html_generator.parse_html_for_line_which_includes_newline(line)?;
        }

        Ok(html! {
          pre class="my-4 py-4 bg-coding_background px-8 overflow-x-auto max-w-vw text-codeText" { code { (PreEscaped(html_generator.finalize())) } }
        })
    }
}

impl IntoHtml for ThematicBreak {
    fn into_html(self, _config: &AppConfig, _context: &MarkdownRenderContext) -> Result<Markup> {
        Ok(html! {
          hr class="my-8 opacity-20";
        })
    }
}
