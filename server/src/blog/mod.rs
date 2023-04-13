use include_dir::{include_dir, Dir};
use markdown::{mdast::*, to_mdast, ParseOptions};
use maud::Markup;
use serde::{Deserialize, Serialize};

static BLOG_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../blog");

pub(crate) struct BlogPosts {
    dir: &'static Dir<'static>,
}

impl BlogPosts {
    pub fn new() -> Self {
        Self { dir: &BLOG_DIR }
    }

    pub fn posts(&self) -> impl Iterator<Item = BlogPostPath> {
        self.dir
            .find("**/*.md")
            .unwrap()
            .map(|entry| BlogPostPath::new(entry.path().to_string_lossy().into_owned()))
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
