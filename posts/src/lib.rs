use std::{path::PathBuf, str::FromStr};

use color_eyre::eyre::Context;
use include_dir::File;
use markdown::{
    mdast::{Node, Root},
    to_mdast, ParseOptions,
};
use serde::Deserialize;

use color_eyre::Result;

use self::{blog::PostMarkdown, date::PostedOn, title::Title};

pub mod blog;
pub mod til;

pub mod date;
pub mod title;

pub mod plain;

pub mod projects;

#[derive(Debug, Clone)]
pub struct Post<FrontmatterType> {
    pub frontmatter: FrontmatterType,
    pub ast: MarkdownAst,
    pub path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct MarkdownAst(pub Root);

impl FromStr for MarkdownAst {
    type Err = color_eyre::Report;

    fn from_str(contents: &str) -> Result<Self> {
        let mut options: ParseOptions = ParseOptions::default();
        options.constructs.gfm_footnote_definition = true;
        options.constructs.gfm_table = true;
        options.constructs.frontmatter = true;

        match to_mdast(contents, &options) {
            Ok(Node::Root(ast)) => Ok(Self(ast)),
            Ok(_) => Err(color_eyre::eyre::eyre!("Should be a root node")),
            Err(e) => Err(color_eyre::eyre::eyre!(
                "Could not make AST. Inner Error: {}",
                e
            )),
        }
    }
}

impl MarkdownAst {
    pub fn from_file(file: &File) -> Result<Self> {
        let contents = file.contents();
        let contents = std::str::from_utf8(contents).wrap_err("File is not UTF8")?;

        Self::from_str(contents)
    }

    fn frontmatter_yml(&self) -> Result<&str> {
        let children = &self.0.children;
        let Some(Node::Yaml(frontmatter)) = children.first() else {
            return Err(color_eyre::eyre::eyre!(
                "Should have a first child with YAML Frontmatter"
            ));
        };

        Ok(&frontmatter.value)
    }

    pub fn frontmatter<T>(&self) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let yaml = self.frontmatter_yml()?;
        serde_yaml::from_str(yaml).wrap_err("Frontmatter should be valid YAML")
    }
}

impl<FrontMatter> Post<FrontMatter>
where
    FrontMatter: PostedOn + Title,
{
    pub fn markdown(&self) -> PostMarkdown {
        PostMarkdown {
            title: self.title().to_string(),
            date: self.posted_on().to_string(),
            ast: self.ast.clone(),
        }
    }
}
