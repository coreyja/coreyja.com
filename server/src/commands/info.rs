use miette::Result;
use posts::blog::BlogPosts;
use syntect::parsing::SyntaxSet;

pub(crate) fn print_info() -> Result<()> {
    println!("\n\n");
    println!("Blog Posts:");

    for p in BlogPosts::from_static_dir()?.posts() {
        let title = p.title();
        let path = p.path();
        let date = p.date();

        println!("{title} | {date}: {path:?}");
    }

    println!("\n\n");
    println!("Recognized Syntax:");

    let ps = SyntaxSet::load_defaults_newlines();
    for syntax in ps.syntaxes() {
        println!("{}", syntax.name);
    }

    Ok(())
}
