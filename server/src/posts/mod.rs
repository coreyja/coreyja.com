use std::path::PathBuf;

use chrono::{DateTime, NaiveTime, Utc};
use include_dir::File;
use markdown::{
    mdast::{Node, Root},
    to_mdast, ParseOptions,
};
use miette::{Context, IntoDiagnostic, Result};
use serde::Deserialize;

use crate::{
    http_server::pages::blog::md::{IntoHtml, IntoPlainText},
    AppState,
};

use self::{
    blog::{LinkTo, PostMarkdown, ToCanonicalPath},
    date::PostedOn,
    title::Title,
};

pub(crate) mod blog;
pub(crate) mod til;

pub(crate) mod date;
pub(crate) mod title;

#[derive(Debug, Clone)]
pub(crate) struct Post<FrontmatterType> {
    pub(crate) frontmatter: FrontmatterType,
    pub(crate) ast: MarkdownAst,
    pub(crate) path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct MarkdownAst(pub(crate) Root);

impl MarkdownAst {
    pub fn from_file(file: &File) -> Result<Self> {
        let contents = file.contents();
        let contents = std::str::from_utf8(contents)
            .into_diagnostic()
            .wrap_err("File is not UTF8")?;

        Self::from_str(contents)
    }

    pub fn from_str(contents: &str) -> Result<Self> {
        let mut options: ParseOptions = Default::default();
        options.constructs.gfm_footnote_definition = true;
        options.constructs.frontmatter = true;

        match to_mdast(contents, &options) {
            Ok(Node::Root(ast)) => Ok(Self(ast)),
            Ok(_) => Err(miette::miette!("Should be a root node")),
            Err(e) => Err(miette::miette!("Could not make AST. Inner Error: {}", e)),
        }
    }

    fn frontmatter_yml(&self) -> Result<&str> {
        let children = &self.0.children;
        let Some(Node::Yaml(frontmatter)) = children.get(0) else {
          return Err(miette::miette!("Should have a first child with YAML Frontmatter"))
        };

        Ok(&frontmatter.value)
    }

    pub fn frontmatter<T>(&self) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let yaml = self.frontmatter_yml()?;
        serde_yaml::from_str(yaml)
            .into_diagnostic()
            .wrap_err("Frontmatter should be valid YAML")
    }
}

impl IntoHtml for MarkdownAst {
    fn into_html(self, state: &AppState) -> maud::Markup {
        self.0.into_html(state)
    }
}

impl<FrontMatter> Post<FrontMatter> {
    fn short_description(&self) -> Option<String> {
        let contents = self.ast.0.plain_text();

        Some(contents.chars().take(100).collect())
    }
}

pub(crate) trait ToRssItem {
    fn to_rss_item(&self, state: &AppState) -> rss::Item;
}

impl<FrontMatter> ToRssItem for Post<FrontMatter>
where
    FrontMatter: PostedOn + Title,
    Post<FrontMatter>: LinkTo,
{
    fn to_rss_item(&self, state: &AppState) -> rss::Item {
        let link = state.app.app_url(&self.link());

        let posted_on: DateTime<Utc> = self.posted_on().and_time(NaiveTime::MIN).and_utc();
        let formatted_date = posted_on.to_rfc2822();

        rss::ItemBuilder::default()
            .title(Some(self.title().to_string()))
            .link(Some(link))
            .description(self.short_description())
            .pub_date(Some(formatted_date))
            .content(Some(self.markdown().ast.0.into_html(state).into_string()))
            .build()
    }
}

impl<FrontMatter> Post<FrontMatter>
where
    FrontMatter: PostedOn + Title,
{
    pub(crate) fn markdown(&self) -> PostMarkdown {
        PostMarkdown {
            title: self.title().to_string(),
            date: self.posted_on().to_string(),
            ast: self.ast.clone(),
        }
    }
}
