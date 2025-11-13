use cja::color_eyre::eyre::Context;

#[derive(Debug, Clone)]
pub struct AnthropicConfig {
    pub api_key: String,
}

impl AnthropicConfig {
    #[tracing::instrument(name = "AnthropicConfig::from_env")]
    pub fn from_env() -> cja::Result<Self> {
        Ok(Self {
            api_key: std::env::var("ANTHROPIC_API_KEY")
                .context("ANTHROPIC_API_KEY env var missing")?,
        })
    }
}
