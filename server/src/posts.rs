use std::path::PathBuf;

use include_dir::File;
use markdown::{
    mdast::{Node, Root},
    to_mdast, ParseOptions,
};
use miette::{Context, IntoDiagnostic, Result};
use serde::Deserialize;

use crate::http_server::pages::blog::md::IntoHtml;

pub(crate) mod blog;
pub(crate) mod til;

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
    fn into_html(
        self,
        context: &crate::http_server::pages::blog::md::HtmlRenderContext,
    ) -> maud::Markup {
        self.0.into_html(context)
    }
}
