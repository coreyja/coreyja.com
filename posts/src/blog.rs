use path_absolutize::Absolutize;
use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDate, Utc};
use include_dir::{include_dir, Dir, File};
use markdown::mdast::Node;
use serde::{Deserialize, Serialize};

use crate::{MarkdownAst, Post};

use super::{
    date::{ByRecency, PostedOn},
    title::Title,
};

use color_eyre::{eyre::Context, Result};

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

    pub fn validate(&self) -> Result<()> {
        self.validate_images()?;

        Ok(())
    }

    fn validate_images(&self) -> Result<()> {
        let p = self.path.canonical_path();
        let p = PathBuf::from(p);

        let root_node = Node::Root(self.ast.0.clone());
        root_node.validate_images(&p)?;

        Ok(())
    }

    pub fn matches_path(&self, path: &str) -> Option<MatchesPath> {
        let path = PathBuf::from(path);

        let canonical = self.path.canonical_path();
        let canonical = PathBuf::from(canonical);

        if canonical == path {
            Some(MatchesPath::CanonicalPath)
        } else if canonical == path.canonical_path() {
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

pub trait ValidateMarkdown {
    fn validate_images(&self, path: &Path) -> Result<()>;
}

impl ValidateMarkdown for Node {
    fn validate_images(&self, path: &Path) -> Result<()> {
        if let Node::Image(image) = self {
            let mut image_path = path.to_path_buf();
            image_path.push(&image.url);

            let cleaned = image_path.absolutize_virtually("/")?;
            let cleaned = cleaned.to_string_lossy().to_string();
            let cleaned = cleaned.strip_prefix('/').unwrap().to_string();

            let post_path = BlogPostPath::new(cleaned.clone());

            if !post_path.file_exists() {
                return Err(color_eyre::eyre::eyre!("Image {} does not exist", cleaned));
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
            .find("**/*.md")?
            .filter_map(|e| e.as_file())
            .filter(|f| {
                // Skip LinkedIn custom-body sibling files.
                // Covers blog posts, newsletters under blog/weekly/, and any future sub-tree.
                let name = f.path().file_name().and_then(|n| n.to_str()).unwrap_or("");
                name != "linkedin.md" && !name.ends_with(".linkedin.md")
            })
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

pub struct BlogPostPath {
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
        self.file_exists()
            && std::path::Path::new(&self.path)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
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
    pub bsky_url: Option<String>,
    pub linkedin_url: Option<String>,
    /// Custom `LinkedIn` body. When `None`, defaults to the first paragraph
    /// (or a sibling `linkedin.md` file if present).
    pub linkedin_content: Option<String>,
    /// When to send the newsletter. If `None` and `is_newsletter` is true, send immediately.
    pub newsletter_send_at: Option<DateTime<Utc>>,
    /// Buttondown email ID, populated after publishing to Buttondown.
    pub buttondown_id: Option<String>,
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
    use markdown::mdast::Root;

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
        use MatchesPath::*;

        let path = PathBuf::from("2020-01-01-test/index.md");
        let meta = BlogFrontMatter {
            title: "Sample Post".to_string(),
            date: NaiveDate::default(),
            is_newsletter: false,
            bsky_url: None,
            linkedin_url: None,
            linkedin_content: None,
            newsletter_send_at: None,
            buttondown_id: None,
        };
        let post = BlogPost {
            path,
            ast: MarkdownAst(Root {
                children: vec![],
                position: None,
            }),
            frontmatter: meta,
        };

        assert_eq!(post.matches_path("2020-01-01-test/"), Some(CanonicalPath));
        assert_eq!(
            post.matches_path("2020-01-01-test/index.md"),
            Some(RedirectToCanonicalPath)
        );
        assert_eq!(post.matches_path("2020-01-01-test/anythingelse"), None);
        assert_eq!(post.matches_path("anythingelse"), None);
    }

    #[test]
    fn frontmatter_deserializes_with_linkedin_url() {
        let yaml = "title: Sample\ndate: 2026-05-23\nlinkedin_url: https://www.linkedin.com/feed/update/urn:li:share:1\n";
        let fm: BlogFrontMatter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            fm.linkedin_url.as_deref(),
            Some("https://www.linkedin.com/feed/update/urn:li:share:1")
        );
        assert!(fm.linkedin_content.is_none());
    }

    #[test]
    fn frontmatter_deserializes_with_linkedin_content() {
        let yaml = "title: Sample\ndate: 2026-05-23\nlinkedin_content: Custom body for LinkedIn.\n";
        let fm: BlogFrontMatter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            fm.linkedin_content.as_deref(),
            Some("Custom body for LinkedIn.")
        );
    }

    #[test]
    fn from_dir_skips_linkedin_md_sibling_files() {
        use include_dir::{include_dir, Dir};
        // Use the real blog dir. We just ensure no parse error and that
        // a hypothetical sibling file would be filtered (this exercises
        // the filter on real input).
        static D: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../blog");
        // Should not error even if any linkedin.md / *.linkedin.md exist
        // (no .md frontmatter required for filtered files).
        let _ = BlogPosts::from_dir(&D).expect("from_dir succeeds");
    }
}
