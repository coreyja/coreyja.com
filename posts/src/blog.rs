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

    /// Slug used for the OG card route. Strips `weekly/` for newsletter posts and trims
    /// the trailing slash from the canonical path.
    pub fn og_slug(&self) -> String {
        let canonical = self.path.canonical_path();
        let trimmed = canonical.trim_end_matches('/');
        if self.frontmatter.is_newsletter {
            trimmed.trim_start_matches("weekly/").to_string()
        } else {
            trimmed.to_string()
        }
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
    /// When to send the newsletter. If `None` and `is_newsletter` is true, send immediately.
    pub newsletter_send_at: Option<DateTime<Utc>>,
    /// Buttondown email ID, populated after publishing to Buttondown.
    pub buttondown_id: Option<String>,
    /// Absolute URL of an OG image to use instead of the auto-generated branded card.
    pub og_image: Option<String>,
    /// Optional short subtitle/tagline shown on the OG card in place of the
    /// post date. When unset, the OG card renders the post date.
    pub subtitle: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub author: Option<String>,
    /// AT URI of the `site.standard.document` record on the PDS, set after
    /// the first successful sync. Acts as the idempotency key — its presence
    /// means the document record has been created, so subsequent syncs use
    /// `putRecord` against the same rkey rather than creating a new record.
    pub atproto_uri: Option<String>,
    /// Publication this post belongs to. Defaults to `"blog"` and corresponds
    /// to an entry in `publications.toml`.
    #[serde(default = "default_publication")]
    pub publication: String,
}

fn default_publication() -> String {
    "blog".to_string()
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
            newsletter_send_at: None,
            buttondown_id: None,
            og_image: None,
            subtitle: None,
            tags: vec![],
            author: None,
            atproto_uri: None,
            publication: "blog".to_string(),
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

    fn test_post(path: &str, is_newsletter: bool) -> BlogPost {
        BlogPost {
            path: PathBuf::from(path),
            ast: MarkdownAst(Root {
                children: vec![],
                position: None,
            }),
            frontmatter: BlogFrontMatter {
                title: "T".to_string(),
                date: NaiveDate::default(),
                is_newsletter,
                bsky_url: None,
                newsletter_send_at: None,
                buttondown_id: None,
                og_image: None,
                subtitle: None,
                tags: vec![],
                author: None,
                atproto_uri: None,
                publication: "blog".to_string(),
            },
        }
    }

    #[test]
    fn og_slug_regular_post() {
        let post = test_post("battlesnake-in-2026/index.md", false);
        assert_eq!(post.og_slug(), "battlesnake-in-2026");
    }

    #[test]
    fn og_slug_newsletter() {
        let post = test_post("weekly/20230713/index.md", true);
        assert_eq!(post.og_slug(), "20230713");
    }

    #[test]
    fn og_slug_weekly_path_but_not_newsletter() {
        let post = test_post("weekly/20230713/index.md", false);
        assert_eq!(post.og_slug(), "weekly/20230713");
    }

    #[test]
    fn frontmatter_defaults_apply_when_fields_absent() {
        let yaml = "title: T\ndate: 2026-05-01";
        let fm: BlogFrontMatter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(fm.publication, "blog");
        assert!(fm.atproto_uri.is_none());
    }
}
