use miette::Result;

use crate::blog::BlogPosts;

pub(crate) async fn print_info() -> Result<()> {
    println!("\n\n");
    println!("Blog Posts:");

    for p in BlogPosts::from_static_dir()?.posts() {
        let title = p.title();
        let path = p.path();
        let date = p.date();

        println!("{title} | {date}: {path:?}");
    }

    Ok(())
}
