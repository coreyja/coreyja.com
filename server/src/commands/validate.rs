use miette::Result;

use crate::blog::BlogPosts;

pub(crate) async fn validate() -> Result<()> {
    let posts = BlogPosts::from_static_dir()?;

    for post in posts.posts() {
        println!("Validating {}...", post.path().display());
        post.validate()?;
    }

    println!("Valid! ✅");
    Ok(())
}
