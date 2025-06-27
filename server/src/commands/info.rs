use cja::Result;
use posts::blog::BlogPosts;

use crate::http_server::pages::blog::md::SyntaxHighlightingContext;

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
    println!("Recognized Syntax:");

    let context = SyntaxHighlightingContext::default();
    let ps = context.syntax_set;
    for syntax in ps.syntaxes() {
        println!("{}", syntax.name);
    }

    Ok(())
}
