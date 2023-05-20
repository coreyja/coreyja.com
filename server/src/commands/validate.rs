use std::println;

use miette::{IntoDiagnostic, Result};
use rss::validation::Validate;

use crate::{blog::BlogPosts, http_server::pages::blog::generate_rss, AppConfig};

pub(crate) async fn validate() -> Result<()> {
    let posts = BlogPosts::from_static_dir()?;

    println!("Validating {} posts", posts.posts().len());
    for post in posts.posts() {
        println!("Validating {}...", post.path().display());
        post.validate()?;
    }
    println!("Posts Valid! ✅");

    println!("Validating RSS feed...");
    let config = AppConfig::from_env()?;
    let rss = generate_rss(config, &posts);
    rss.validate().into_diagnostic()?;

    println!("RSS Valid! ✅");

    Ok(())
}
