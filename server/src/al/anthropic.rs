use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct AnthropicTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct AnthropicRequest {
    pub model: String,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub messages: Vec<Message>,
    pub tools: Vec<AnthropicTool>,
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
}

#[derive(Debug, Serialize)]
pub struct ThinkingConfig {
    pub r#type: String,
    pub budget_tokens: u32,
}

#[derive(Debug, Serialize)]
pub struct ToolChoice {
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: Vec<Content>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheControl {
    pub r#type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AnthropicResponse {
    pub content: Vec<Content>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Content {
    #[serde(rename = "text")]
    Text(TextContent),
    #[serde(rename = "image")]
    Image(ImageContent),
    #[serde(rename = "document")]
    Document(DocumentContent),
    #[serde(rename = "tool_use")]
    ToolUse(ToolUseContent),
    #[serde(rename = "tool_result")]
    ToolResult(ToolResult),
    #[serde(rename = "thinking")]
    Thinking(ThinkingContent),
}

impl Content {
    pub fn set_cache_control(&mut self, cache_control: CacheControl) {
        match self {
            Content::Text(text_content) => {
                text_content.cache_control = Some(cache_control);
            }
            Content::Image(image_content) => {
                image_content.cache_control = Some(cache_control);
            }
            Content::Document(document_content) => {
                document_content.cache_control = Some(cache_control);
            }
            Content::ToolUse(tool_use_content) => {
                tool_use_content.cache_control = Some(cache_control);
            }
            Content::ToolResult(tool_result) => {
                tool_result.cache_control = Some(cache_control);
            }
            Content::Thinking(thinking_content) => {
                thinking_content.cache_control = Some(cache_control);
            }
        }
    }

    pub fn cache_control(&self) -> Option<&CacheControl> {
        match self {
            Content::Text(text_content) => text_content.cache_control.as_ref(),
            Content::Image(image_content) => image_content.cache_control.as_ref(),
            Content::Document(document_content) => document_content.cache_control.as_ref(),
            Content::ToolUse(tool_use_content) => tool_use_content.cache_control.as_ref(),
            Content::ToolResult(tool_result) => tool_result.cache_control.as_ref(),
            Content::Thinking(thinking_content) => thinking_content.cache_control.as_ref(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TextContent {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ImageContent {
    pub source: ImageSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ImageSource {
    pub r#type: String, // "base64" or "url"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>, // "image/jpeg", "image/png", "image/gif", "image/webp"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>, // base64-encoded image data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>, // URL to image
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct DocumentContent {
    pub source: DocumentSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct DocumentSource {
    pub r#type: String, // "base64" or "url"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>, // "application/pdf"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>, // base64-encoded document data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>, // URL to document
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ToolUseContent {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ToolResult {
    pub tool_use_id: String,
    pub content: String,
    pub is_error: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ThinkingContent {
    pub thinking: String,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}
