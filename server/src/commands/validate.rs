use std::println;

use miette::{IntoDiagnostic, Result};
use rss::validation::Validate;

use crate::{
    http_server::pages::blog::{generate_rss, md::HtmlRenderContext},
    posts::{blog::BlogPosts, til::TilPosts},
    AppConfig, AppState,
};

pub(crate) async fn validate() -> Result<()> {
    let posts = BlogPosts::from_static_dir()?;

    println!("Validating {} posts", posts.posts().len());
    for post in posts.posts() {
        println!("Validating {}...", post.path().display());
        post.validate()?;
    }
    println!("Posts Valid! ✅");

    {
        let tils = TilPosts::from_static_dir()?;

        tils.validate()?;
    }

    println!("Validating RSS feed...");
    let config = AppConfig::from_env()?;
    let render_context = HtmlRenderContext::default();

    let rss = generate_rss(config, render_context, &posts);
    rss.validate().into_diagnostic()?;

    println!("RSS Valid! ✅");

    Ok(())
}
