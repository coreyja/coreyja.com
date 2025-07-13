use rig::providers::anthropic;

pub mod standup;

pub fn create_anthropic_client() -> cja::Result<anthropic::Client> {
    let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
        cja::color_eyre::eyre::eyre!("ANTHROPIC_API_KEY environment variable not set")
    })?;

    Ok(anthropic::client::ClientBuilder::new(&api_key).build())
}
