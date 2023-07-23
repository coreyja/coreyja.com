use miette::{Context, Result};
use path_absolutize::Absolutize;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use include_dir::{include_dir, Dir, File};
use markdown::mdast::*;
use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};

use crate::{
    http_server::pages::blog::md::{HtmlRenderContext, IntoHtml},
    posts::{MarkdownAst, Post},
    AppConfig,
};

use super::{
    date::{ByRecency, PostedOn},
    title::Title,
};

static BLOG_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../blog");

#[derive(Debug)]
pub(crate) struct BlogPosts {
    posts: Vec<BlogPost>,
}

pub(crate) type BlogPost = Post<BlogFrontMatter>;

impl BlogPost {
    fn from_file(file: &File) -> Result<BlogPost> {
        let ast = MarkdownAst::from_file(file)?;

        let metadata: BlogFrontMatter = ast.frontmatter()?;

        let path = file.path().to_owned();

        Ok(BlogPost {
            frontmatter: metadata,
            path,
            ast,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn title(&self) -> &str {
        &self.frontmatter.title
    }

    pub fn ast(&self) -> &MarkdownAst {
        &self.ast
    }

    pub fn date(&self) -> &NaiveDate {
        &self.frontmatter.date
    }

    pub(crate) fn validate(&self) -> Result<()> {
        self.validate_images()?;

        Ok(())
    }

    fn validate_images(&self) -> Result<()> {
        let p = self.canonical_path();
        let p = PathBuf::from(p);

        let root_node = Node::Root(self.ast.0.clone());
        root_node.validate_images(&p)?;

        Ok(())
    }

    pub fn matches_path(&self, path: &str) -> Option<MatchesPath> {
        let path = PathBuf::from(path);

        let canonical = self.canonical_path();
        let canonical = PathBuf::from(canonical);

        if canonical == path {
            Some(MatchesPath::CanonicalPath)
        } else if canonical == PathBuf::from(path.canonical_path()) {
            Some(MatchesPath::RedirectToCanonicalPath)
        } else {
            None
        }
    }

    pub(crate) fn to_rss_item(
        &self,
        config: &AppConfig,
        render_context: &HtmlRenderContext,
    ) -> rss::Item {
        let link = config.app_url(&self.canonical_path());

        let formatted_date = self.frontmatter.date.to_string();

        rss::ItemBuilder::default()
            .title(Some(self.frontmatter.title.clone()))
            .link(Some(link))
            .description(self.short_description())
            .pub_date(Some(formatted_date))
            .content(Some(
                self.markdown()
                    .ast
                    .0
                    .into_html(render_context)
                    .into_string(),
            ))
            .build()
    }
}

#[derive(PartialEq, Debug)]
pub enum MatchesPath {
    CanonicalPath,
    RedirectToCanonicalPath,
}

pub trait ValidateMarkdown {
    fn validate_images(&self, path: &Path) -> Result<()>;
}

impl ValidateMarkdown for Node {
    fn validate_images(&self, path: &Path) -> Result<()> {
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

    pub(crate) fn by_recency(&self) -> Vec<&BlogPost> {
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

        let ast = MarkdownAst::from_file(file).expect("Should be able to parse markdown");
        let metadata: BlogFrontMatter = ast.frontmatter().unwrap();

        Some(PostMarkdown {
            title: metadata.title,
            date: metadata.date.to_string(),
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
    pub ast: MarkdownAst,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub(crate) struct BlogFrontMatter {
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

impl<T> ToCanonicalPath for Post<T> {
    fn canonical_path(&self) -> String {
        self.path.canonical_path()
    }
}

impl<T> ToCanonicalPath for &Post<T> {
    fn canonical_path(&self) -> String {
        self.path.canonical_path()
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
