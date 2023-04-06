use axum::extract::Path;
use include_dir::{include_dir, Dir};
use maud::{html, Markup};

static BLOG_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../blog");

pub async fn post_get(Path(key): Path<String>) -> Markup {
    let file = BLOG_DIR.get_file(key);

    for file in BLOG_DIR.files() {
        println!("{}", file.path().display());
    }

    if file.is_none() {
        return html! {
            h1 { "404" }
        };
    }
    let file = file.unwrap();
    let contents = file.contents_utf8().unwrap();

    html! {
      p { (contents) }
      h1 { "Hello World"  }
    }
}
