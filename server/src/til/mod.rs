use std::path::PathBuf;

use include_dir::{include_dir, Dir, File};
use markdown::mdast::Node;
use miette::{Context, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};

use crate::{
    blog::{PostMarkdown, ValidateMarkdown},
    posts::{MarkdownAst, Post},
};

pub(crate) static TIL_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../til");

#[derive(Debug)]
pub(crate) struct TilPosts {
    pub(crate) posts: Vec<TilPost>,
}

type TilPost = Post<FrontMatter>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub(crate) struct FrontMatter {
    pub title: String,
    pub date: String,
    pub slug: String,
}

impl TilPost {
    fn from_file(file: &File) -> Result<Self> {
        let ast = MarkdownAst::from_file(file)?;
        let metadata: FrontMatter = ast.frontmatter()?;
        let path = file.path().to_owned();

        Ok(Self {
            ast,
            path,
            frontmatter: metadata,
        })
    }

    pub(crate) fn validate(&self) -> Result<()> {
        self.validate_images()?;

        Ok(())
    }

    fn validate_images(&self) -> Result<()> {
        let p = &self.frontmatter.slug;
        let p = PathBuf::from(p);

        let root_node = Node::Root(self.ast.0.clone());
        root_node.validate_images(&p)?;

        Ok(())
    }

    pub(crate) fn markdown(&self) -> PostMarkdown {
        PostMarkdown {
            title: self.frontmatter.title.clone(),
            date: self.frontmatter.date.to_string(),
            ast: self.ast.clone(),
        }
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
}
