use axum::extract::Path;
use include_dir::{include_dir, Dir, File};
use markdown::{
    mdast::{Node, Root},
    to_html, to_mdast, ParseOptions,
};
use maud::{html, Markup, PreEscaped};
use serde::{Deserialize, Serialize};

static BLOG_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../blog");

pub async fn post_get(Path(key): Path<String>) -> Markup {
    let Some(markdown) = BlogPostPath::new(key).to_markdown() else {
      return html! {
        h1 { "404" }
      };
    };

    html! {
      h1 { (markdown.title) }
      subtitle { (markdown.date) }

      (markdown.html)
    }
}

struct BlogPostPath {
    path: String,
}

impl BlogPostPath {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub fn to_markdown(&self) -> Option<PostMarkdown> {
        let file = BLOG_DIR.get_file(&self.path)?;

        let contents = file.contents_utf8().expect("All posts are UTF8");

        let mut options: ParseOptions = Default::default();
        options.constructs.frontmatter = true;

        let Node::Root(ast) = to_mdast(contents, &options).unwrap() else {
          panic!("Should be a root node")
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
            html: html! { (PreEscaped(to_html(contents))) },
        })
    }
}

struct PostMarkdown {
    title: String,
    date: String,
    ast: Root,
    // TODO: Stop using the html here and actually parse the ast above to convert to HTML
    html: Markup,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
struct FrontMatter {
    title: String,
    date: String,
}
