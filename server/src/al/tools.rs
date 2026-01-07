use db::agentic_threads::Thread;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

use crate::{
    al::anthropic::{AnthropicTool, ToolUseContent},
    AppState,
};
pub mod cooking_simple;
pub mod discord;
pub mod linear_graphql;
pub mod memory;
pub mod threads;
pub mod tool_suggestions;

#[derive(Debug, Clone)]
pub struct ThreadContext {
    pub thread: Thread,
    pub previous_stitch_id: Option<Uuid>,
}

#[async_trait::async_trait]
pub trait Tool: Send + Sync + Sized + 'static {
    const NAME: &'static str;
    const DESCRIPTION: &'static str;

    type ToolInput: Serialize + for<'a> Deserialize<'a> + JsonSchema;
    type ToolOutput: Serialize + for<'a> Deserialize<'a>;

    fn tool_parameters(&self) -> serde_json::Value {
        let schema = schemars::schema_for!(Self::ToolInput);
        serde_json::to_value(schema).unwrap()
    }

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput>;

    fn to_generic(self) -> Box<dyn GenericTool> {
        Box::new(self)
    }
}

#[async_trait::async_trait]
pub trait GenericTool: Sync + Send {
    fn tool_name(&self) -> &'static str;
    fn tool_description(&self) -> &'static str;
    fn tool_parameters(&self) -> serde_json::Value;

    async fn run(
        &self,
        input: serde_json::Value,
        app_state: AppState,
        context: ThreadContext,
    ) -> cja::Result<serde_json::Value, GenericToolError>;
}

#[derive(Debug, thiserror::Error)]
pub enum GenericToolError {
    #[error("Tool not found: {0}")]
    NotFound(String),
    #[error("Tool input error: {0}")]
    InputError(serde_json::Error),
    #[error("Tool output error: {0}")]
    OutputError(serde_json::Error),
    #[error("Tool error: {0}")]
    Error(#[from] cja::color_eyre::eyre::Report),
}

#[async_trait::async_trait]
impl<T: Tool + Sync + Send> GenericTool for T {
    fn tool_name(&self) -> &'static str {
        Self::NAME
    }

    fn tool_description(&self) -> &'static str {
        Self::DESCRIPTION
    }

    fn tool_parameters(&self) -> serde_json::Value {
        self.tool_parameters()
    }

    async fn run(
        &self,
        input: serde_json::Value,
        app_state: AppState,
        context: ThreadContext,
    ) -> cja::Result<serde_json::Value, GenericToolError> {
        let input: T::ToolInput =
            serde_json::from_value(input).map_err(GenericToolError::InputError)?;
        let output = self.run(input, app_state, context).await?;
        Ok(serde_json::to_value(output).map_err(GenericToolError::OutputError)?)
    }
}

#[derive(Default)]
pub struct ToolBag {
    tools_by_name: std::collections::BTreeMap<String, Box<dyn GenericTool>>,
}

impl ToolBag {
    pub fn add_generic_tool(&mut self, tool: Box<dyn GenericTool>) -> cja::Result<&mut Self> {
        let existing = self.tools_by_name.get(tool.tool_name());
        if existing.is_some() {
            color_eyre::eyre::bail!("Tool with name {} already exists", tool.tool_name());
        }

        self.tools_by_name
            .insert(tool.tool_name().to_string(), tool);
        Ok(self)
    }

    pub fn add_tool(&mut self, tool: impl Tool) -> cja::Result<&mut Self> {
        self.add_generic_tool(tool.to_generic())
    }

    /// Add tools from an agent configuration, filtered by thread type.
    /// This will iterate through the enabled tools and add only those that are
    /// appropriate for the given thread type (Interactive vs Autonomous).
    pub fn add_tools_from_config(
        &mut self,
        config: &crate::agent_config::AgentConfig,
        thread_type: &db::agentic_threads::ThreadType,
    ) -> cja::Result<&mut Self> {
        for tool in &config.enabled_tools {
            // Only add tools that are appropriate for this thread type
            if tool.is_available_for_thread_type(thread_type) {
                self.add_generic_tool(tool.create_instance())?;
            }
        }
        Ok(self)
    }

    pub(crate) fn as_api(&self) -> Vec<AnthropicTool> {
        self.tools_by_name
            .values()
            .map(|tool| AnthropicTool {
                name: tool.tool_name().to_string(),
                description: tool.tool_description().to_string(),
                input_schema: tool.tool_parameters(),
            })
            .collect()
    }

    pub(crate) async fn call_tool(
        &self,
        tool_use_content: ToolUseContent,
        app_state: AppState,
        context: ThreadContext,
    ) -> cja::Result<serde_json::Value, GenericToolError> {
        let tool = self
            .tools_by_name
            .get(tool_use_content.name.as_str())
            .ok_or(GenericToolError::NotFound(tool_use_content.name))?;

        let output = tool.run(tool_use_content.input, app_state, context).await?;
        Ok(output)
    }
}
