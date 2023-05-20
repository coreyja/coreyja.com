use miette::{Context, Result};
use path_absolutize::Absolutize;
use std::{
    path::{Path, PathBuf},
    str::from_utf8,
};

use chrono::NaiveDate;
use include_dir::{include_dir, Dir, File};
use markdown::{mdast::*, to_mdast, ParseOptions};
use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};

use crate::AppConfig;

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

    pub(crate) fn validate(&self) -> Result<()> {
        self.validate_images()?;

        Ok(())
    }

    fn validate_images(&self) -> Result<()> {
        let p = self.canonical_path();
        let p = PathBuf::from(p);

        let root_node = Node::Root(self.ast.clone());
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

    pub(crate) fn markdown(&self) -> PostMarkdown {
        PostMarkdown {
            title: self.title.clone(),
            date: self.date.to_string(),
            ast: self.ast.clone(),
        }
    }

    pub(crate) fn to_rss_item(&self, config: &AppConfig) -> rss::Item {
        let link = config.app_url(&self.canonical_path());

        rss::ItemBuilder::default()
            .title(Some(self.title.clone()))
            .link(Some(link))
            .description(self.short_description())
            .build()
    }

    fn short_description(&self) -> Option<String> {
        let file = BLOG_DIR.get_file(&self.path)?;

        let contents = from_utf8(file.contents()).unwrap();

        // Find the start of the frontmatter
        let frontmatter_marks = contents.find("---");

        let contents = if let Some(frontmatter_marks) = frontmatter_marks {
            let (_, contents) = contents.split_at(frontmatter_marks + 3);
            contents
        } else {
            contents
        };

        // Find the end of the frontmatter
        let frontmatter_marks = contents.find("---");

        let contents = if let Some(frontmatter_marks) = frontmatter_marks {
            let (_, contents) = contents.split_at(frontmatter_marks + 3);
            contents
        } else {
            contents
        };

        Some(contents.chars().take(100).collect())
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

impl ToCanonicalPath for BlogPost {
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
        let post = BlogPost {
            path,
            title: "Sample Post".to_string(),
            ast: Root {
                children: vec![],
                position: None,
            },
            date: Default::default(),
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
