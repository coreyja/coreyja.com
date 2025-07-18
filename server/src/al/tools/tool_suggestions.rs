use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{al::tools::Tool, AppState};
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

    async fn run(&self, input: Self::ToolInput) -> cja::Result<Self::ToolOutput> {
        // Validate input
        if input.name.trim().is_empty() {
            return Ok(ToolSuggestionOutput {
                success: false,
                message: "Tool name cannot be empty".to_string(),
            });
        }

        if input.description.trim().is_empty() {
            return Ok(ToolSuggestionOutput {
                success: false,
                message: "Tool description cannot be empty".to_string(),
            });
        }

        if input.examples.is_empty() {
            return Ok(ToolSuggestionOutput {
                success: false,
                message: "At least one example is required".to_string(),
            });
        }

        // Convert examples to JSON array
        let examples_json = serde_json::json!(input.examples);

        // Try to create the suggestion
        match ToolSuggestion::create(
            &self.app_state.db,
            input.name,
            input.description,
            examples_json,
        )
        .await
        {
            Ok(_) => Ok(ToolSuggestionOutput {
                success: true,
                message: "Tool suggestion submitted successfully! The team will review it and consider implementation.".to_string(),
            }),
            Err(e) => {
                // Check if it's a duplicate name error
                if e.to_string().contains("duplicate key value") {
                    Ok(ToolSuggestionOutput {
                        success: false,
                        message: "A tool suggestion with this name already exists".to_string(),
                    })
                } else {
                    Err(e)
                }
            }
        }
    }
}
