use miette::Result;

use crate::*;

pub(crate) mod completion;
pub(crate) mod edit;

#[derive(Debug, Clone)]
pub(crate) struct OpenAiConfig {
    pub api_key: String,
}

impl OpenAiConfig {
    #[instrument]
    pub fn from_env() -> Result<Self> {
        let open_ai_api_key = std::env::var("OPEN_AI_API_KEY")
            .into_diagnostic()
            .wrap_err("No OpenAI API KEY Found")?;

        Ok(Self {
            api_key: open_ai_api_key,
        })
    }
}
