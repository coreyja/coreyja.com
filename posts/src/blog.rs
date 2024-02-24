use miette::{Context, Result};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use chrono::NaiveDate;
use include_dir::{include_dir, Dir, File};
use markdown::mdast::*;
use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};

use crate::{MarkdownAst, Post};

use super::{
    date::{ByRecency, PostedOn},
    title::Title,
};

#[cfg(feature = "data")]
static BLOG_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../blog");

#[derive(Debug, Clone)]
pub struct BlogPosts {
    posts: Vec<BlogPost>,
}

pub type BlogPost = Post<BlogFrontMatter>;

impl BlogPost {
    pub(crate) fn from_file(file: &File) -> Result<BlogPost> {
        let ast = MarkdownAst::from_file(file)?;

        let metadata: BlogFrontMatter = ast.frontmatter()?;

        let path = file.path().to_owned();
        let raw_markdown: String = file
            .contents_utf8()
            .ok_or_else(|| miette::miette!("No markdown found"))?
            .to_string();

        Ok(BlogPost {
            frontmatter: metadata,
            path,
            raw_markdown,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn title(&self) -> &str {
        &self.frontmatter.title
    }

    pub fn date(&self) -> &NaiveDate {
        &self.frontmatter.date
    }

    #[cfg(feature = "data")]
    pub fn validate(&self) -> Result<()> {
        self.validate_images()?;

        Ok(())
    }

    #[cfg(feature = "data")]
    fn validate_images(&self) -> Result<()> {
        let p = self.path.canonical_path();
        let p = PathBuf::from(p);

        let root_node = Node::Root(self.ast().0.clone());
        root_node.validate_images(&p)?;

        Ok(())
    }

    pub fn matches_path(&self, path: &str) -> Option<MatchesPath> {
        let path = PathBuf::from(path);

        let canonical = self.path.canonical_path();
        let canonical = PathBuf::from(canonical);

        if canonical == path {
            Some(MatchesPath::CanonicalPath)
        } else if canonical == PathBuf::from(path.canonical_path()) {
            Some(MatchesPath::RedirectToCanonicalPath)
        } else {
            None
        }
    }

    // pub(crate) fn to_rss_item(&self, state: &AppState) -> rss::Item {
    //     let link = state.app.app_url(&self.canonical_path());

    //     let formatted_date = self.frontmatter.date.to_string();

    //     rss::ItemBuilder::default()
    //         .title(Some(self.frontmatter.title.clone()))
    //         .link(Some(link))
    //         .description(self.short_description())
    //         .pub_date(Some(formatted_date))
    //         .content(Some(self.markdown().ast.0.into_html(state).into_string()))
    //         .build()
    // }
}

#[derive(PartialEq, Debug)]
pub enum MatchesPath {
    CanonicalPath,
    RedirectToCanonicalPath,
}

#[cfg(feature = "data")]
pub trait ValidateMarkdown {
    fn validate_images(&self, path: &Path) -> Result<()>;
}

#[cfg(feature = "data")]
impl ValidateMarkdown for Node {
    fn validate_images(&self, path: &Path) -> Result<()> {
        use path_absolutize::Absolutize;

        if let Node::Image(image) = self {
            let mut image_path = path.to_path_buf();
            image_path.push(&image.url);

            let cleaned = image_path.absolutize_virtually("/").into_diagnostic()?;
            let cleaned = cleaned.to_string_lossy().to_string();
            let cleaned = cleaned.strip_prefix('/').unwrap().to_string();

            let post_path = BlogPostPath::new(cleaned.clone());

            if !post_path.file_exists() {
                return Err(miette::miette!("Image {} does not exist", cleaned));
            }

            Ok(())
        } else {
            if let Some(children) = self.children() {
                for child in children {
                    child.validate_images(path)?;
                }
            }

            Ok(())
        }
    }
}

impl BlogPosts {
    #[cfg(feature = "data")]
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

    pub fn by_recency(&self) -> Vec<&BlogPost> {
        self.posts.by_recency()
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

#[cfg(feature = "data")]
pub struct BlogPostPath {
    pub path: String,
}

#[cfg(feature = "data")]
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

    pub fn raw_bytes(&self) -> &'static [u8] {
        let file = BLOG_DIR.get_file(&self.path).unwrap();

        file.contents()
    }
}

pub struct PostMarkdown {
    pub title: String,
    pub date: String,
    pub ast: MarkdownAst,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct BlogFrontMatter {
    pub title: String,
    pub date: NaiveDate,
    #[serde(default = "default_is_newsletter")]
    pub is_newsletter: bool,
}

impl PostedOn for BlogFrontMatter {
    fn posted_on(&self) -> chrono::NaiveDate {
        self.date
    }
}

impl Title for BlogFrontMatter {
    fn title(&self) -> &str {
        &self.title
    }
}

fn default_is_newsletter() -> bool {
    false
}

pub trait ToCanonicalPath {
    fn canonical_path(&self) -> String;
}

impl ToCanonicalPath for PathBuf {
    fn canonical_path(&self) -> String {
        let c = self.clone();

        if c.file_name() == Some(std::ffi::OsStr::new("index.md")) {
            return format!("{}/", c.parent().unwrap().to_string_lossy());
        }

        if c.extension() == Some(std::ffi::OsStr::new("md")) {
            let mut c = c;

            c.set_extension("");

            return format!("{}/", c.to_string_lossy());
        }

        format!("{}", self.to_string_lossy())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_canonical_path_named_markdown() {
        let path = PathBuf::from("2020-01-01-test.md");

        assert_eq!(path.canonical_path(), "2020-01-01-test/");
    }

    #[test]
    fn test_canonical_path_index_markdown() {
        let path = PathBuf::from("2020-01-01-test/index.md");

        assert_eq!(path.canonical_path(), "2020-01-01-test/");
    }

    #[test]
    fn test_canonical_path_dir() {
        let path = PathBuf::from("2020-01-01-test/");

        assert_eq!(path.canonical_path(), "2020-01-01-test/");
    }

    #[test]
    fn test_path_matching() {
        let path = PathBuf::from("2020-01-01-test/index.md");
        let meta = BlogFrontMatter {
            title: "Sample Post".to_string(),
            date: Default::default(),
            is_newsletter: false,
        };
        let post = BlogPost {
            path,
            ast: MarkdownAst(Root {
                children: vec![],
                position: None,
            }),
            frontmatter: meta,
        };

        use MatchesPath::*;

        assert_eq!(post.matches_path("2020-01-01-test/"), Some(CanonicalPath));
        assert_eq!(
            post.matches_path("2020-01-01-test/index.md"),
            Some(RedirectToCanonicalPath)
        );
        assert_eq!(post.matches_path("2020-01-01-test/anythingelse"), None);
        assert_eq!(post.matches_path("anythingelse"), None);
    }
}
