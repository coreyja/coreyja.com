use std::println;

use miette::{IntoDiagnostic, Result};

use crate::{
    http_server::pages::blog::{md::SyntaxHighlightingContext, MyChannel},
    posts::{blog::BlogPosts, til::TilPosts},
    AppConfig,
};

pub(crate) async fn validate() -> Result<()> {
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
    let config = AppConfig::from_env()?;
    let render_context = SyntaxHighlightingContext::default();

    let rss = MyChannel::from_posts(config, render_context, &posts.by_recency());

    rss.validate().into_diagnostic()?;

    println!("Blog RSS Valid! ✅");

    println!("Validating TIL RSS feed...");
    let config = AppConfig::from_env()?;
    let render_context = SyntaxHighlightingContext::default();

    let rss = MyChannel::from_posts(config, render_context, &tils.by_recency());

    rss.validate().into_diagnostic()?;

    println!("TIL RSS Valid! ✅");

    Ok(())
}
