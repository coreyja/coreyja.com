use chrono::NaiveDate;
use include_dir::{include_dir, Dir, File};
use markdown::{
    mdast::{Node, Root},
    to_mdast, ParseOptions,
};
use miette::{Context, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};

pub(crate) static TIL_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../til");

#[derive(Debug)]
pub(crate) struct TilPosts {
    pub(crate) posts: Vec<TilPost>,
}

#[derive(Debug, Clone)]
pub struct TilPost {
    pub(crate) title: String,
    pub(crate) date: NaiveDate,
    pub(crate) slug: String,
    pub(crate) ast: Root,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
struct FrontMatter {
    pub title: String,
    pub date: String,
    pub slug: String,
}

impl TilPost {
    fn from_file(file: &File) -> Result<Self> {
        let contents = file.contents();
        let contents = std::str::from_utf8(contents)
            .into_diagnostic()
            .wrap_err("File is not UTF8")?;

        let mut options: ParseOptions = Default::default();
        options.constructs.gfm_footnote_definition = true;
        options.constructs.frontmatter = true;

        let Ok(Node::Root(ast)) = to_mdast(contents, &options) else {
      return Err(miette::miette!("Should be a valid root node"));
    };

        let children = &ast.children;
        let Some(Node::Yaml(frontmatter)) = children.get(0) else {
      return Err(miette::miette!("Should have a child with YAML Frontmatter"))
    };

        let yaml = &frontmatter.value;

        let metadata: FrontMatter = serde_yaml::from_str(yaml)
            .into_diagnostic()
            .wrap_err("Frontmatter should be valid YAML")?;

        let title = metadata.title;
        let date = metadata
            .date
            .parse::<NaiveDate>()
            .into_diagnostic()
            .wrap_err_with(|| format!("Date should be valid: {}", metadata.date))?;

        Ok(Self {
            title,
            ast,
            date,
            slug: metadata.slug,
        })
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
            .wrap_err("One of the TIL posts failed to parse")?;

        Ok(Self { posts })
    }
}
