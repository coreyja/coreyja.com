use tracing::instrument;

pub mod chat;
pub mod completion;
pub mod edit;

use miette::{Context, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct OpenAiConfig {
    pub api_key: String,
}

impl OpenAiConfig {
    #[instrument(name = "OpenAiConfig::from_env")]
    pub fn from_env() -> Result<Self> {
        let open_ai_api_key = std::env::var("OPEN_AI_API_KEY")
            .into_diagnostic()
            .wrap_err("No OpenAI API KEY Found")?;

        Ok(Self {
            api_key: open_ai_api_key,
        })
    }
}
