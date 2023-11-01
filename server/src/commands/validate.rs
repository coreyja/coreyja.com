use std::{println, sync::Arc};

use miette::{IntoDiagnostic, Result};
use openai::OpenAiConfig;
use posts::{blog::BlogPosts, past_streams::PastStreams, projects::Projects, til::TilPosts};

use crate::{
    github::GithubConfig,
    http_server::pages::blog::{md::SyntaxHighlightingContext, MyChannel},
    twitch::TwitchConfig,
    AppConfig, AppState,
};

pub(crate) async fn validate() -> Result<()> {
    let projects = Projects::from_static_dir()?;
    println!("Validating {} projects", projects.projects.len());
    for project in projects.projects {
        println!("Validating {}...", project.frontmatter.title);
        project.validate()?;
    }

    let posts = BlogPosts::from_static_dir()?;

    println!("Validating {} posts", posts.posts().len());
    for post in posts.posts() {
        println!("Validating {}...", post.path().display());
        post.validate()?;
    }
    println!("Posts Valid! ✅");

    let tils = TilPosts::from_static_dir()?;

    tils.validate()?;

    println!("Validating Past Streams feed...");
    let streams = PastStreams::from_static_dir()?;
    streams.validate()?;

    println!("Validating Blog RSS feed...");
    let config = AppConfig::from_env()?;
    let render_context = SyntaxHighlightingContext::default();
    // TODO: This is a bit of a hack, but it's fine for now.
    // I need a better way to either have default values for these
    // or allow them to be a `None` value.
    let state = AppState {
        twitch: TwitchConfig {
            client_id: "".to_string(),
            client_secret: "".to_string(),
            bot_access_token: None,
            channel_user_id: "".to_string(),
            bot_user_id: "".to_string(),
        },
        github: GithubConfig {
            app_id: 0,
            client_id: "".to_string(),
            client_secret: "".to_string(),
        },
        open_ai: OpenAiConfig {
            api_key: "".to_string(),
        },
        blog_posts: Arc::new(posts.clone()),
        til_posts: Arc::new(tils.clone()),
        streams: Arc::new(streams.clone()),
        app: config,
        markdown_to_html_context: render_context,
    };

    let rss = MyChannel::from_posts(state.clone(), &posts.by_recency());

    rss.validate().into_diagnostic()?;

    println!("Blog RSS Valid! ✅");

    println!("Validating TIL RSS feed...");

    let rss = MyChannel::from_posts(state, &tils.by_recency());

    rss.validate().into_diagnostic()?;

    println!("TIL RSS Valid! ✅");

    Ok(())
}
