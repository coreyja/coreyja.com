use std::println;

use posts::{blog::BlogPosts, projects::Projects, til::TilPosts};

use crate::{
    http_server::pages::blog::{md::SyntaxHighlightingContext, MyChannel},
    AppConfig,
};

pub(crate) fn validate() -> cja::Result<()> {
    let projects = Projects::from_static_dir()?;
    projects.validate()?;

    let posts = BlogPosts::from_static_dir()?;
    println!("Validating {} posts", posts.posts().len());
    for post in posts.posts() {
        println!("Validating {}...", post.path().display());
        post.validate()?;
    }
    println!("Posts Valid! ✅");

    let tils = TilPosts::from_static_dir()?;

    tils.validate()?;

    println!("Validating Blog RSS feed...");
    let config = AppConfig {
        base_url: "http://localhost:3000".to_string(),
    };
    let render_context = SyntaxHighlightingContext::default();
    let rss = MyChannel::from_posts(&config, &render_context, &posts.by_recency())?;

    rss.validate()?;

    println!("Blog RSS Valid! ✅");

    println!("Validating TIL RSS feed...");

    let rss = MyChannel::from_posts(&config, &render_context, &tils.by_recency())?;

    rss.validate()?;

    println!("TIL RSS Valid! ✅");

    Ok(())
}
