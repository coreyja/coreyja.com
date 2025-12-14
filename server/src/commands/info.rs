use cja::Result;
use posts::blog::BlogPosts;

pub(crate) fn print_info() -> Result<()> {
    println!("\n\n");
    println!("Blog Posts:");

    for p in BlogPosts::from_static_dir()?.posts() {
        let title = p.title();
        let path = p.path();
        let date = p.date();

        println!("{title} | {date}: {}", path.display());
    }

    println!("\n\n");
    println!("Syntax Highlighting: arborium (Tree-sitter based)");
    println!(
        "Enabled languages: rust, bash, javascript, ruby, graphql, toml, json, css, html, vim"
    );

    Ok(())
}
