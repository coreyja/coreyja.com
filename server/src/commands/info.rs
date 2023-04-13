use miette::Result;

use crate::blog::BlogPosts;

pub(crate) async fn print_info() -> Result<()> {
    println!("\n\n");
    println!("Blog Posts:");
    for p in BlogPosts::new().posts() {
        let Some(post) = p.to_markdown() else {
        continue;
      };

        let title = post.title;
        let url = p.path;

        println!("{title}: {url}");
    }

    Ok(())
}
