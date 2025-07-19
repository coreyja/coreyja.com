use color_eyre::eyre::bail;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    al::tools::{ThreadContext, Tool},
    AppState,
};
use db::tool_suggestions::ToolSuggestion;

#[derive(Clone, Debug)]
pub struct ToolSuggestionsSubmit {
    app_state: AppState,
}

impl ToolSuggestionsSubmit {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolExample {
    /// The input parameters for this example
    pub input: serde_json::Value,
    /// The expected output for this example
    pub output: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolSuggestionInput {
    /// The name of the suggested tool
    pub name: String,
    /// A description of what the tool should do
    pub description: String,
    /// Examples showing how the tool would be called and what it would return
    pub examples: Vec<ToolExample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSuggestionOutput {
    pub success: bool,
    pub message: String,
}

#[async_trait::async_trait]
impl Tool for ToolSuggestionsSubmit {
    const NAME: &'static str = "tool_suggestions";
    const DESCRIPTION: &'static str = "Submit a suggestion for a new tool that would be helpful for agents. Include a clear name, description, and examples of inputs/outputs to demonstrate the tool's functionality.";

    type ToolInput = ToolSuggestionInput;
    type ToolOutput = ToolSuggestionOutput;

    async fn run(
        &self,
        input: Self::ToolInput,
        context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        let Some(previous_stitch_id) = context.previous_stitch_id else {
            bail!("Tool suggestion requires a previous stitch in the thread");
        };

        // Validate input
        if input.name.trim().is_empty() {
            bail!("Tool name cannot be empty");
        }

        if input.description.trim().is_empty() {
            bail!("Tool description cannot be empty");
        }

        if input.examples.is_empty() {
            bail!("At least one example is required");
        }

        // Convert examples to JSON array
        let examples_json = serde_json::json!(input.examples);

        // Try to create the suggestion
        ToolSuggestion::create(
            &self.app_state.db,
            input.name,
            input.description,
            examples_json,
            previous_stitch_id,
        )
        .await.map(|_| ToolSuggestionOutput {
            success: true,
            message: "Tool suggestion submitted successfully! The team will review it and consider implementation.".to_string(),
        })
    }
}
