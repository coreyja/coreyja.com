use std::path::PathBuf;
use std::unreachable;

use markdown::mdast::{
    Blockquote, Break, Code, Delete, Emphasis, Heading, Html, Image, InlineCode, Link, List,
    ListItem, Node, Paragraph, Root, Strong, Table, TableCell, TableRow, Text, ThematicBreak, Yaml,
};
use maud::{html, Markup, PreEscaped};
use url::Url;

use color_eyre::Result;

use crate::AppConfig;

use urlencoding::encode;

fn generate_imgproxy_url(base_url: &str, image_url: &str, width: u32) -> String {
    format!(
        "{}/unsafe/rs:fit:{width}:0/plain/{}",
        base_url,
        encode(image_url)
    )
}

/// Syntax highlighting context using arborium (Tree-sitter based highlighting).
/// This is kept as an empty struct for backwards compatibility with existing code.
#[derive(Debug, Clone, Default)]
pub struct SyntaxHighlightingContext;

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
            Node::Blockquote(x) => x.into_html(config, context),
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

impl IntoHtml for Blockquote {
    fn into_html(self, config: &AppConfig, context: &MarkdownRenderContext) -> Result<Markup> {
        Ok(html! {
          blockquote {
            (self.children.into_html(config, context)?)
          }
        })
    }
}

impl IntoHtml for Text {
    fn into_html(self, _config: &AppConfig, _context: &MarkdownRenderContext) -> Result<Markup> {
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
        #[allow(unreachable_code)]
        let inner = html! {
            @match self.depth {
                1 => h1 id=[id] class="max-w-prose text-2xl" { (child_content) },
                2 => h2 id=[id] class="max-w-prose text-xl" { (child_content) },
                3 => h3 id=[id] class="max-w-prose text-lg" { (child_content) },
                4 => h4 id=[id] class="max-w-prose text-lg text-subtitle" { (child_content) },
                5 => h5 id=[id] class="max-w-prose text-lg text-subtitle font-light" { (child_content) },
                6 => h6 id=[id] class="max-w-prose text-base text-subtitle" { (child_content) },
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
            } @else {
                ul class="max-w-prose" { (inner) }
            }
        })
    }
}

impl IntoHtml for InlineCode {
    fn into_html(self, _config: &AppConfig, _context: &MarkdownRenderContext) -> Result<Markup> {
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

        let img_tag: Result<Markup> = if let Some(imgproxy_base) = config.imgproxy_url.as_ref() {
            let img_src = config.app_url(&relative_url);

            let small_url = generate_imgproxy_url(imgproxy_base, &img_src, 600);
            let med_url = generate_imgproxy_url(imgproxy_base, &img_src, 900);
            let large_url = generate_imgproxy_url(imgproxy_base, &img_src, 1200);

            let srcset = format!("{small_url}, {med_url} 1.5x, {large_url} 2x");

            Ok(html! {
                img srcset=(srcset) src=(small_url) alt=(self.alt) title=[self.title] loading="lazy" {}
            })
        } else {
            Ok(html! {
                img src=(relative_url) alt=(self.alt) title=[self.title] loading="lazy" {}
            })
        };
        let img_tag = img_tag?;

        Ok(html! {
            figure class="px-8 my-8" {
                (img_tag)
                figcaption class="px-4 py-1 text-sm italic" {
                    (self.alt)
                }
            }
        })
    }
}

impl IntoHtml for Link {
    fn into_html(self, config: &AppConfig, context: &MarkdownRenderContext) -> Result<Markup> {
        let parsed_base = config.base_url.clone();
        let parsed_base_host = parsed_base.host_str();

        let parse_options = Url::options().base_url(Some(&parsed_base));
        let url = parse_options.parse(&self.url);

        let is_external = url
            .as_ref()
            .is_ok_and(|url| url.host_str() != parsed_base_host);

        let replaced_url = url.map_or_else(|_| self.url.clone(), |url| url.to_string());

        Ok(html! {
            @if is_external {
                a href=(replaced_url) title=[self.title] class="underline" target="_blank" rel="noopener noreferrer" { (self.children.into_html(config, context)?) }
            } @else {
                a href=(replaced_url) title=[self.title] class="underline" { (self.children.into_html(config, context)?) }
            }
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
    fn into_html(self, _config: &AppConfig, _context: &MarkdownRenderContext) -> Result<Markup> {
        let lang = self.lang.as_deref().unwrap_or("text");

        // Map language tokens to arborium-supported languages
        let mapped_lang = match lang {
            "shell" => "bash",
            "gitignore" | "crontab" | "fen" => "text",
            other => other,
        };

        let highlighted_html = arborium::Highlighter::new()
            .highlight(mapped_lang, &self.value)
            .unwrap_or_else(|_| html_escape::encode_text(&self.value).to_string());

        Ok(html! {
          pre class="my-4 py-4 bg-coding_background px-8 overflow-x-auto max-w-vw text-codeText" { code { (PreEscaped(highlighted_html)) } }
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
