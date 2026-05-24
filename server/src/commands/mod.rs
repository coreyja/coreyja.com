use cja::Result;
use clap::Subcommand;

pub(crate) mod bluesky;
pub(crate) mod buttondown;
pub(crate) mod info;
pub(crate) mod linkedin;
pub(crate) mod validate;

#[derive(Subcommand, Default)]
pub(crate) enum Command {
    #[default]
    Serve,
    Print,
    Validate,
    /// Publish a newsletter to Buttondown
    PublishButtondown(buttondown::PublishButtondownArgs),
    /// Publish a note to Bluesky
    PublishBluesky(bluesky::PublishBlueskyArgs),
    /// Publish to `LinkedIn`
    #[command(name = "publish-linkedin", about = "Publish to LinkedIn")]
    PublishLinkedin(linkedin::PublishLinkedInArgs),
}

impl Command {
    pub(crate) async fn run(&self) -> Result<()> {
        match &self {
            Command::Serve => crate::http_server::cmd::serve().await,
            Command::Print => info::print_info(),
            Command::Validate => validate::validate(),
            Command::PublishButtondown(args) => buttondown::publish_buttondown(args).await,
            Command::PublishBluesky(args) => bluesky::publish_bluesky(args).await,
            Command::PublishLinkedin(args) => linkedin::publish_linkedin(args).await,
        }
    }
}
