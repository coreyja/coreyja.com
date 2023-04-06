use axum::{extract::Path, http::StatusCode};
use include_dir::{include_dir, Dir};
use markdown::{
    mdast::{Node, Root},
    to_html, to_mdast, ParseOptions,
};
use maud::{html, Markup, PreEscaped};
use serde::{Deserialize, Serialize};

static BLOG_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../blog");

pub async fn post_get(Path(mut key): Path<String>) -> Result<Markup, StatusCode> {
    // TODO: Eventually
    //
    // I think we can move away from the wildcard route and instead
    // use the static-ness of BLOG_DIR to setup all the routes on server
    // boot.
    // Thay way we can static routes to route the different posts and avoid the wildcard
    // This might make it easier to do something like generate a sitemap from the routes
    key = key.strip_suffix('/').unwrap_or(&key).to_string();
    key = key.strip_suffix("index.md").unwrap_or(&key).to_string();

    let mut path = BlogPostPath::new(key.clone());

    if !path.file_exists() {
        path = BlogPostPath::new(format!("{key}.md"));
    }

    if !path.file_exists() {
        path = BlogPostPath::new(format!("{key}/index.md"));
    }

    let Some(markdown) = path.to_markdown() else {
      return Err(StatusCode::NOT_FOUND);
    };

    Ok(html! {
      h1 { (markdown.title) }
      subtitle { (markdown.date) }

      (markdown.html)
    })
}

struct BlogPostPath {
    path: String,
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