use color_eyre::Result;

use crate::*;

#[derive(Debug, Clone)]
pub(crate) struct OpenAiConfig {
    pub api_key: String,
}

impl OpenAiConfig {
    pub fn from_env() -> Result<Self> {
        let open_ai_api_key =
            std::env::var("OPEN_AI_API_KEY").wrap_err("No OpenAI API KEY Found")?;

        Ok(Self {
            api_key: open_ai_api_key,
        })
    }
}

mod completion;
pub(crate) use completion::*;

mod edit;
#[allow(unused_imports)]
pub(crate) use edit::*;
