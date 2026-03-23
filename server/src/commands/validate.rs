use std::println;

use posts::{blog::BlogPosts, notes::NotePosts, projects::Projects};
use url::Url;

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

    let notes = NotePosts::from_static_dir()?;

    notes.validate()?;

    println!("Validating Blog RSS feed...");
    let config = AppConfig {
        base_url: Url::parse("http://localhost:3000").unwrap(),
        imgproxy_url: None,
    };
    let render_context = SyntaxHighlightingContext;
    let rss = MyChannel::from_posts(&config, &render_context, &posts.by_recency())?;

    rss.validate()?;

    println!("Blog RSS Valid! ✅");

    println!("Validating Notes RSS feed...");

    let rss = MyChannel::from_posts(&config, &render_context, &notes.by_recency())?;

    rss.validate()?;

    println!("Notes RSS Valid! ✅");

    Ok(())
}
