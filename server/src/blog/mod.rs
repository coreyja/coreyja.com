use miette::{Context, Result};
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use include_dir::{include_dir, Dir, File};
use markdown::{mdast::*, to_mdast, ParseOptions};
use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};

static BLOG_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../blog");

pub(crate) struct BlogPosts {
    posts: Vec<BlogPost>,
}

#[derive(Debug, Clone)]
pub struct BlogPost {
    path: PathBuf,
    title: String,
    ast: Root,
    date: NaiveDate,
}

impl BlogPost {
    fn from_file(file: &File) -> Result<BlogPost> {
        println!("We made it this far {:?}", file.path());

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
            .wrap_err("Frontmatter should be valid JSON")?;

        let title = metadata.title;
        let path = file.path().to_owned();
        let date = metadata
            .date
            .parse::<NaiveDate>()
            .into_diagnostic()
            .wrap_err_with(|| format!("Date should be valid: {}", metadata.date))?;

        Ok(BlogPost {
            path,
            title,
            ast,
            date,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn ast(&self) -> &Root {
        &self.ast
    }

    pub fn date(&self) -> &NaiveDate {
        &self.date
    }
}

impl BlogPosts {
    pub fn from_static_dir() -> Result<Self> {
        Self::from_dir(&BLOG_DIR)
    }

    pub fn from_dir(dir: &Dir) -> Result<Self> {
        let posts = dir
            .find("**/*.md")
            .into_diagnostic()?
            .filter_map(|e| e.as_file())
            .map(BlogPost::from_file)
            .collect::<Result<Vec<_>>>()
            .wrap_err("One of the blog posts failed to parse")?;

        Ok(Self { posts })
    }

    pub fn posts(&self) -> &Vec<BlogPost> {
        &self.posts
    }
}

pub struct BlogFileEntry<'a> {
    entry: &'a File<'static>,
}

impl<'a> BlogFileEntry<'a> {
    pub fn path(&self) -> &'a Path {
        self.entry.path()
    }
}

pub(crate) struct BlogPostPath {
    pub path: String,
}

impl BlogPostPath {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub fn file_exists(&self) -> bool {
        BLOG_DIR.get_file(&self.path).is_some()
    }

    pub fn file_is_markdown(&self) -> bool {
        self.file_exists() && self.path.ends_with(".md")
    }

    pub fn to_markdown(&self) -> Option<PostMarkdown> {
        let file = BLOG_DIR.get_file(&self.path)?;

        let contents = file.contents_utf8().expect("All posts are UTF8");

        let mut options: ParseOptions = Default::default();
        options.constructs.gfm_footnote_definition = true;
        options.constructs.frontmatter = true;

        let Ok(Node::Root(ast)) = to_mdast(contents, &options) else {
          panic!("Should be a valid root node")
        };

        let children = &ast.children;
        let Node::Yaml(frontmatter) = children.get(0).expect("Should have frontmatter") else {
          panic!("Should have a YAML Frontmatter")
        };

        let yaml = &frontmatter.value;

        let metadata: FrontMatter = serde_yaml::from_str(yaml).expect("Should be valid YAML");

        Some(PostMarkdown {
            title: metadata.title,
            date: metadata.date,
            ast,
        })
    }

    pub(crate) fn raw_bytes(&self) -> &'static [u8] {
        let file = BLOG_DIR.get_file(&self.path).unwrap();

        file.contents()
    }
}

pub(crate) struct PostMarkdown {
    pub title: String,
    pub date: String,
    pub ast: Root,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
struct FrontMatter {
    pub title: String,
    pub date: String,
}
