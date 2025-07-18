use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::al::standup::{AnthropicTool, ToolUseContent};
pub mod discord;

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

    async fn run(&self, input: Self::ToolInput) -> cja::Result<Self::ToolOutput>;

    fn to_generic(self) -> Box<dyn GenericTool> {
        Box::new(self)
    }
}

#[async_trait::async_trait]
pub trait GenericTool: Sync + Send {
    fn tool_name(&self) -> &'static str;
    fn tool_description(&self) -> &'static str;
    fn tool_parameters(&self) -> serde_json::Value;

    async fn run(&self, input: serde_json::Value) -> cja::Result<serde_json::Value>;
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

    async fn run(&self, input: serde_json::Value) -> cja::Result<serde_json::Value> {
        let input: T::ToolInput = serde_json::from_value(input)?;
        let output = self.run(input).await?;
        Ok(serde_json::to_value(output)?)
    }
}

#[derive(Default)]
pub struct ToolBag {
    tools_by_name: std::collections::HashMap<String, Box<dyn GenericTool>>,
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
    ) -> cja::Result<serde_json::Value> {
        let tool = self
            .tools_by_name
            .get(tool_use_content.name.as_str())
            .ok_or_else(|| {
                cja::color_eyre::eyre::eyre!("Tool not found: {}", tool_use_content.name)
            })?;

        let output = tool.run(tool_use_content.input).await?;
        Ok(output)
    }
}
