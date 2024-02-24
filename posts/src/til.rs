use std::path::PathBuf;

use chrono::NaiveDate;
use include_dir::{include_dir, Dir, File};
use markdown::mdast::Node;
use miette::{Context, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};

use crate::{MarkdownAst, Post};

use super::{
    date::{ByRecency, PostedOn},
    title::Title,
};

pub(crate) static TIL_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../til");

#[derive(Debug, Clone)]
pub struct TilPosts {
    pub posts: Vec<TilPost>,
}

pub type TilPost = Post<FrontMatter>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct FrontMatter {
    pub title: String,
    pub date: NaiveDate,
    pub slug: String,
}

impl PostedOn for FrontMatter {
    fn posted_on(&self) -> chrono::NaiveDate {
        self.date
    }
}

impl Title for FrontMatter {
    fn title(&self) -> &str {
        &self.title
    }
}

impl TilPost {
    fn from_file(file: &File) -> Result<Self> {
        let ast = MarkdownAst::from_file(file)?;
        let metadata: FrontMatter = ast.frontmatter()?;
        let path = file.path().to_owned();

        let raw_markdown: String = file
            .contents_utf8()
            .ok_or_else(|| miette::miette!("No markdown found"))?
            .to_string();

        Ok(Self {
            raw_markdown,
            path,
            frontmatter: metadata,
        })
    }

    #[cfg(feature = "data")]
    pub(crate) fn validate(&self) -> Result<()> {
        self.validate_images()?;

        Ok(())
    }

    #[cfg(feature = "data")]
    fn validate_images(&self) -> Result<()> {
        use crate::blog::ValidateMarkdown;

        let p = &self.frontmatter.slug;
        let p = PathBuf::from(p);

        let root_node = Node::Root(self.ast().0.clone());
        root_node.validate_images(&p)?;

        Ok(())
    }
}

impl TilPosts {
    pub fn from_static_dir() -> Result<Self> {
        Self::from_dir(&TIL_DIR)
    }

    pub fn from_dir(dir: &Dir) -> Result<Self> {
        let posts = dir
            .find("**/*.md")
            .into_diagnostic()?
            .filter_map(|e| e.as_file())
            .map(TilPost::from_file)
            .collect::<Result<Vec<_>>>()
            .wrap_err("One of the TILs failed to parse")?;

        Ok(Self { posts })
    }

    #[cfg(feature = "data")]
    pub fn validate(&self) -> Result<()> {
        println!("Validating Slug Uniqueness");
        for slug in self.posts.iter().map(|til| &til.frontmatter.slug) {
            let matches: Vec<_> = self
                .posts
                .iter()
                .filter(|til| &til.frontmatter.slug == slug)
                .collect();
            if matches.len() > 1 {
                let paths = matches
                    .iter()
                    .map(|til| til.path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(miette::miette!(
                    "Slug {} is not unique. Found these paths {}",
                    slug,
                    paths
                ));
            }
        }

        println!("Validating {} TILs", self.posts.len());
        for til in &self.posts {
            println!(
                "Validating {} from {}...",
                til.frontmatter.slug,
                til.path.display()
            );

            til.validate()?;
        }
        println!("TILs Valid! ✅");

        Ok(())
    }

    pub fn by_recency(&self) -> Vec<&TilPost> {
        self.posts.by_recency()
    }
}
