use cja::jobs::Job;
use color_eyre::eyre::bail;
use db::agentic_threads::{Stitch, Thread, ThreadStatus};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{types::Uuid, PgPool};

use crate::{
    al::{
        standup::{
            AnthropicRequest, AnthropicResponse, CacheControl, Content, DocumentContent,
            DocumentSource, ImageContent, ImageSource, Message, TextContent, ToolChoice,
            ToolResult,
        },
        tools::{ThreadContext, ToolBag},
    },
    AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessThreadStep {
    pub thread_id: Uuid,
}

#[async_trait::async_trait]
impl Job<AppState> for ProcessThreadStep {
    const NAME: &'static str = "ProcessThreadStep";

    async fn run(&self, app_state: AppState) -> cja::Result<()> {
        process_single_step(&app_state, self.thread_id).await?;

        let thread = Thread::get_by_id(&app_state.db, self.thread_id)
            .await?
            .ok_or_else(|| cja::color_eyre::eyre::eyre!("Thread not found"))?;

        if thread.status == ThreadStatus::Running {
            ProcessThreadStep {
                thread_id: self.thread_id,
            }
            .enqueue(
                app_state.clone(),
                "Thread processing continuation".to_string(),
            )
            .await?;
        }

        Ok(())
    }
}

#[allow(clippy::too_many_lines)]
async fn process_single_step(app_state: &AppState, thread_id: Uuid) -> cja::Result<()> {
    let thread = Thread::get_by_id(&app_state.db, thread_id)
        .await?
        .ok_or_else(|| cja::color_eyre::eyre::eyre!("Thread not found"))?;

    match thread.status {
        ThreadStatus::Completed | ThreadStatus::Failed | ThreadStatus::Aborted => {
            return Ok(());
        }
        ThreadStatus::Pending | ThreadStatus::Waiting => {
            bail!(
                "Thread is in unexpected state for processing job: {:?}",
                thread.status
            );
        }
        ThreadStatus::Running => {}
    }

    let previous_stitch = Stitch::get_last_stitch(&app_state.db, thread_id).await?;
    let previous_stitch_id = previous_stitch.as_ref().map(|s| s.stitch_id);

    if let Some(s) = previous_stitch {
        match s.stitch_type {
            db::agentic_threads::StitchType::LlmCall => bail!(
                "Right now we only support starting by running an LLM call and then running tool calls after.
                So its not expected to be processing with an LLM call as the previous stitch."
            ),
            db::agentic_threads::StitchType::ThreadResult => todo!(),
            db::agentic_threads::StitchType::InitialPrompt
            | db::agentic_threads::StitchType::ToolCall
            | db::agentic_threads::StitchType::DiscordMessage
            | db::agentic_threads::StitchType::SystemPrompt
            | db::agentic_threads::StitchType::AgentThought
            | db::agentic_threads::StitchType::ClarificationRequest
            | db::agentic_threads::StitchType::Error => {
                // This is the expected types that we can process here right now
            }
        }
    }

    let messages = reconstruct_messages(&app_state.db, thread_id).await?;
    let system_prompt = extract_system_prompt(&app_state.db, thread_id).await?;

    // Get agent configuration - parse agent_name from database to AgentId
    use std::str::FromStr;
    let agent_id = crate::agent_config::AgentId::from_str(&thread.agent_name).map_err(|_| {
        cja::color_eyre::eyre::eyre!(
            "Invalid agent name '{}' for thread {}",
            thread.agent_name,
            thread_id
        )
    })?;

    let agent_config = agent_id.config();

    // Set up tools based on agent configuration and thread type
    // The agent config contains the list of enabled tools, and we automatically
    // add the ones appropriate for this thread type (Interactive vs Autonomous).
    // Interactive threads get Discord thread-specific tools (SendDiscordThreadMessage, ListenToThread, etc.)
    // Autonomous threads get regular tools (SendDiscordMessage, CompleteThread, etc.)
    let mut tools = ToolBag::default();
    tools.add_tools_from_config(&agent_config, &thread.thread_type)?;

    // Make LLM request
    let request = AnthropicRequest {
        model: "claude-sonnet-4-5-20250929".to_string(),
        max_tokens: 1024,
        system: system_prompt,
        messages: messages.clone(),
        tools: tools.as_api(),
        tool_choice: Some(ToolChoice {
            r#type: "any".to_string(),
        }),
        thinking: Some(crate::al::standup::ThinkingConfig {
            r#type: "enabled".to_string(),
            budget_tokens: 10000,
        }),
    };

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &app_state.anthropic.api_key)
        .header("anthropic-version", "2023-06-01")
        .header("anthropic-beta", "pdfs-2024-09-25")
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(cja::color_eyre::eyre::eyre!(
            "Anthropic API error: {}\nRequest: {:?}\nMessages: {:?}",
            error_text,
            request,
            messages,
        ));
    }

    let response_data: AnthropicResponse = response.json().await?;

    let llm_stitch = Stitch::create_llm_call(
        &app_state.db,
        thread_id,
        previous_stitch_id,
        serde_json::to_value(&request)?,
        serde_json::to_value(&response_data)?,
    )
    .await?;

    let mut previous_stitch_id = Some(llm_stitch.stitch_id);

    // Process tool calls
    for content in response_data.content {
        match content {
            Content::Text(_text) => {
                // Text content from assistant - no action needed
            }
            Content::Thinking(_thinking) => {
                // Thinking content from assistant - already stored in llm_response
            }
            Content::ToolUse(tool_use_content) => {
                let tool_name = tool_use_content.name.clone();
                let tool_input = tool_use_content.input.clone();

                // Create thread context with the previous stitch ID
                let context = ThreadContext {
                    thread: thread.clone(),
                    previous_stitch_id,
                };

                let tool_result = tools
                    .call_tool(tool_use_content, app_state.clone(), context)
                    .await;
                let tool_stitch = Stitch::create_tool_call(
                    &app_state.db,
                    thread_id,
                    previous_stitch_id,
                    tool_name,
                    tool_input,
                    tool_result.unwrap_or_else(|e| json!({"error": e.to_string()})),
                )
                .await?;

                previous_stitch_id = Some(tool_stitch.stitch_id);
            }
            Content::ToolResult(_) => {
                unreachable!("ToolResult should not appear in assistant response")
            }
            Content::Image(_) => {
                unreachable!("Image should not appear in assistant response")
            }
            Content::Document(_) => {
                unreachable!("Document should not appear in assistant response")
            }
        }
    }

    Ok(())
}

// Size limits for attachments (based on Anthropic API limits)
const MAX_IMAGE_SIZE: u64 = 5_000_000; // 5MB
const MAX_PDF_SIZE: u64 = 32_000_000; // 32MB

/// Resize an image to fit within the size limit
/// Returns (`resized_bytes`, `media_type`)
fn resize_image_to_limit(img_bytes: &[u8], max_size_bytes: u64) -> cja::Result<(Vec<u8>, String)> {
    use image::ImageFormat;

    // Load the image
    let img = image::load_from_memory(img_bytes)
        .map_err(|e| cja::color_eyre::eyre::eyre!("Failed to decode image: {}", e))?;

    // Start with original dimensions
    let (width, height) = (img.width(), img.height());

    // Try different scaling factors until we get under the limit
    for scale in [1.0, 0.8, 0.6, 0.4, 0.3, 0.2] {
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let new_width = (f64::from(width) * scale) as u32;
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let new_height = (f64::from(height) * scale) as u32;

        if new_width == 0 || new_height == 0 {
            continue;
        }

        let resized = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);

        // Try encoding as JPEG with quality 85
        let mut jpeg_bytes = Vec::new();
        resized
            .write_to(
                &mut std::io::Cursor::new(&mut jpeg_bytes),
                ImageFormat::Jpeg,
            )
            .map_err(|e| cja::color_eyre::eyre::eyre!("Failed to encode image: {}", e))?;

        // Check if it's under the limit
        if jpeg_bytes.len() as u64 <= max_size_bytes {
            tracing::info!(
                "Resized image from {}x{} to {}x{}, {} bytes -> {} bytes",
                width,
                height,
                new_width,
                new_height,
                img_bytes.len(),
                jpeg_bytes.len()
            );
            return Ok((jpeg_bytes, "image/jpeg".to_string()));
        }
    }

    Err(cja::color_eyre::eyre::eyre!(
        "Could not resize image to fit within {} bytes limit",
        max_size_bytes
    ))
}

/// Detect content type from attachment, with fallback to file extension
fn detect_content_type(attachment: &poise::serenity_prelude::Attachment) -> Option<String> {
    // First, try the provided content_type
    if let Some(ct) = &attachment.content_type {
        return Some(ct.clone());
    }

    // Fallback to file extension (case-insensitive)
    let path = std::path::Path::new(&attachment.filename);
    let extension = path.extension()?.to_str()?.to_lowercase();

    match extension.as_str() {
        "jpg" | "jpeg" => Some("image/jpeg".to_string()),
        "png" => Some("image/png".to_string()),
        "gif" => Some("image/gif".to_string()),
        "webp" => Some("image/webp".to_string()),
        "pdf" => Some("application/pdf".to_string()),
        _ => None,
    }
}

/// Helper function to download and encode Discord attachments as base64
async fn process_discord_attachment(
    attachment: &poise::serenity_prelude::Attachment,
) -> cja::Result<Option<Content>> {
    // Detect content type with fallback to file extension
    let content_type = detect_content_type(attachment);

    let is_image = content_type
        .as_ref()
        .is_some_and(|ct| ct.starts_with("image/"));

    let is_pdf = content_type
        .as_ref()
        .is_some_and(|ct| ct == "application/pdf");

    // Only process images and PDFs
    if !is_image && !is_pdf {
        // For other attachments, just mention them in text
        return Ok(None);
    }

    // Check PDF size limit before downloading
    if is_pdf && u64::from(attachment.size) > MAX_PDF_SIZE {
        return Err(cja::color_eyre::eyre::eyre!(
            "PDF '{}' exceeds size limit: {} bytes (max {} bytes / 32MB)",
            attachment.filename,
            attachment.size,
            MAX_PDF_SIZE
        ));
    }

    // Download the attachment with timeout
    let client = reqwest::Client::new();
    let response = client
        .get(&attachment.url)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(cja::color_eyre::eyre::eyre!(
            "Failed to download attachment: {}",
            response.status()
        ));
    }

    let mut bytes = response.bytes().await?.to_vec();
    let mut media_type = content_type.unwrap_or_else(|| {
        if is_pdf {
            "application/pdf".to_string()
        } else {
            "image/png".to_string()
        }
    });

    // If image is too large, resize it
    if is_image && bytes.len() as u64 > MAX_IMAGE_SIZE {
        tracing::info!(
            "Image '{}' is {} bytes, exceeds {}MB limit, resizing...",
            attachment.filename,
            bytes.len(),
            MAX_IMAGE_SIZE / 1_000_000
        );

        match resize_image_to_limit(&bytes, MAX_IMAGE_SIZE) {
            Ok((resized_bytes, new_media_type)) => {
                bytes = resized_bytes;
                media_type = new_media_type;
            }
            Err(e) => {
                return Err(cja::color_eyre::eyre::eyre!(
                    "Failed to resize image '{}': {}",
                    attachment.filename,
                    e
                ));
            }
        }
    }

    let base64_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);

    // Return appropriate content type
    if is_pdf {
        Ok(Some(Content::Document(DocumentContent {
            source: DocumentSource {
                r#type: "base64".to_string(),
                media_type: Some(media_type),
                data: Some(base64_data),
                url: None,
            },
            cache_control: None,
        })))
    } else {
        Ok(Some(Content::Image(ImageContent {
            source: ImageSource {
                r#type: "base64".to_string(),
                media_type: Some(media_type),
                data: Some(base64_data),
                url: None,
            },
            cache_control: None,
        })))
    }
}

pub async fn extract_system_prompt(db: &PgPool, thread_id: Uuid) -> cja::Result<Option<String>> {
    let stitches = Stitch::get_by_thread_ordered(db, thread_id).await?;

    let mut system_prompt = None;

    for stitch in stitches {
        if stitch.stitch_type == db::agentic_threads::StitchType::SystemPrompt {
            if system_prompt.is_some() {
                bail!("Multiple system prompts found in thread");
            }

            if let Some(request) = stitch.llm_request {
                if let Some(text) = request.get("text").and_then(|v| v.as_str()) {
                    system_prompt = Some(text.to_string());
                }
            }
        }
    }

    Ok(system_prompt)
}

#[allow(clippy::too_many_lines)]
pub async fn reconstruct_messages(db: &PgPool, thread_id: Uuid) -> cja::Result<Vec<Message>> {
    let stitches = Stitch::get_by_thread_ordered(db, thread_id).await?;

    let mut messages = Vec::new();
    let mut pending_tool_results: Vec<Content> = Vec::new();

    // Track tool uses from the current LLM call in order
    let mut current_tool_uses: Vec<(String, String)> = Vec::new(); // (tool_name, tool_use_id)
    let mut tool_use_index = 0;

    for stitch in stitches {
        match stitch.stitch_type {
            db::agentic_threads::StitchType::InitialPrompt => {
                if let Some(request) = stitch.llm_request {
                    if let Some(request_messages) =
                        request.get("messages").and_then(|v| v.as_array())
                    {
                        for msg in request_messages {
                            messages.push(serde_json::from_value(msg.clone())?);
                        }
                    }
                }
            }
            db::agentic_threads::StitchType::LlmCall => {
                if !pending_tool_results.is_empty() {
                    messages.push(Message {
                        role: "user".to_string(),
                        content: pending_tool_results.clone(),
                    });
                    pending_tool_results.clear();
                }

                // Reset tool use tracking for this LLM call
                current_tool_uses.clear();
                tool_use_index = 0;

                if let Some(response) = stitch.llm_response {
                    if let Some(content) = response.get("content") {
                        let content_vec: Vec<Content> = serde_json::from_value(content.clone())?;

                        // Collect tool uses in order
                        for content_item in &content_vec {
                            if let Content::ToolUse(tool_use) = content_item {
                                current_tool_uses
                                    .push((tool_use.name.clone(), tool_use.id.clone()));
                            }
                        }

                        messages.push(Message {
                            role: "assistant".to_string(),
                            content: content_vec,
                        });
                    }
                }
            }
            db::agentic_threads::StitchType::ToolCall => {
                if let (Some(tool_name), Some(_tool_input), Some(tool_output)) = (
                    stitch.tool_name.clone(),
                    stitch.tool_input,
                    stitch.tool_output,
                ) {
                    // Find the tool_use_id by matching against the current tool uses in order
                    let tool_use_id = if tool_use_index < current_tool_uses.len() {
                        let (expected_tool_name, tool_id) = &current_tool_uses[tool_use_index];
                        if expected_tool_name == &tool_name {
                            tool_use_index += 1;
                            tool_id.clone()
                        } else {
                            // Fallback if tool name doesn't match expected order
                            format!("tool_{}_{}", tool_name, stitch.stitch_id)
                        }
                    } else {
                        // Fallback if we've exhausted the tool uses
                        format!("tool_{}_{}", tool_name, stitch.stitch_id)
                    };

                    let (content, is_error) = if let Some(err) = tool_output.get("error") {
                        (
                            serde_json::to_string(&err).unwrap_or_else(|_| {
                                "SYSTEM: Tool Errored but can't serialize error".to_string()
                            }),
                            true,
                        )
                    } else {
                        (serde_json::to_string(&tool_output)?, false)
                    };

                    pending_tool_results.push(Content::ToolResult(ToolResult {
                        tool_use_id,
                        content,
                        is_error,
                        cache_control: None,
                    }));
                }
            }
            db::agentic_threads::StitchType::ThreadResult => {
                todo!("Thread results would be handled here when we add child thread support");
            }
            db::agentic_threads::StitchType::SystemPrompt => {
                // Skip system prompts - they're handled separately
            }
            db::agentic_threads::StitchType::DiscordMessage => {
                // First, add any pending tool results before the Discord message
                if !pending_tool_results.is_empty() {
                    messages.push(Message {
                        role: "user".to_string(),
                        content: pending_tool_results.clone(),
                    });
                    pending_tool_results.clear();
                }

                // Handle Discord messages as user messages
                if let Some(request) = stitch.llm_request {
                    if let Some(message_data) = request.get("data") {
                        let message: serenity::all::Message =
                            serde_json::from_value(message_data.clone())?;

                        let mut content_parts = Vec::new();

                        // Format message with user information and message ID
                        let formatted_message = format!(
                            "[{} (@{}, ID: {}, Message ID: {})]: {}",
                            message
                                .author
                                .global_name
                                .as_deref()
                                .unwrap_or(message.author.name.as_str()),
                            message.author.tag(),
                            message.author.id,
                            message.id,
                            message.content
                        );

                        content_parts.push(Content::Text(TextContent {
                            text: formatted_message,
                            cache_control: None,
                        }));

                        // Process attachments
                        for attachment in &message.attachments {
                            match process_discord_attachment(attachment).await {
                                Ok(Some(attachment_content)) => {
                                    content_parts.push(attachment_content);
                                }
                                Ok(None) => {
                                    // Non-image/PDF attachment - mention it in text
                                    let attachment_info = format!(
                                        "\n[Attachment: {} ({} bytes)]",
                                        attachment.filename, attachment.size
                                    );
                                    if let Some(Content::Text(text)) = content_parts.first_mut() {
                                        text.text.push_str(&attachment_info);
                                    }
                                }
                                Err(e) => {
                                    // Log error but continue processing
                                    tracing::warn!(
                                        "Failed to process attachment {}: {}",
                                        attachment.filename,
                                        e
                                    );
                                    let attachment_error = format!(
                                        "\n[Failed to load attachment: {}]",
                                        attachment.filename
                                    );
                                    if let Some(Content::Text(text)) = content_parts.first_mut() {
                                        text.text.push_str(&attachment_error);
                                    }
                                }
                            }
                        }

                        messages.push(Message {
                            role: "user".to_string(),
                            content: content_parts,
                        });
                    }
                }
            }
            db::agentic_threads::StitchType::AgentThought => {
                // Agent thoughts are internal reasoning that should be included in the context
                // but marked appropriately
                if let Some(data) = stitch.llm_request {
                    if let Some(thought) = data.get("thought").and_then(|v| v.as_str()) {
                        messages.push(Message {
                            role: "assistant".to_string(),
                            content: vec![Content::Text(TextContent {
                                text: format!("[AGENT THOUGHT]: {thought}"),
                                cache_control: None,
                            })],
                        });
                    }
                }
            }
            db::agentic_threads::StitchType::ClarificationRequest => {
                // Clarification requests are questions from the agent to the user
                if let Some(data) = stitch.llm_request {
                    if let Some(question) = data.get("question").and_then(|v| v.as_str()) {
                        messages.push(Message {
                            role: "assistant".to_string(),
                            content: vec![Content::Text(TextContent {
                                text: format!("[CLARIFICATION REQUEST]: {question}"),
                                cache_control: None,
                            })],
                        });
                    }
                }
            }
            db::agentic_threads::StitchType::Error => {
                // Error states should be included to show what went wrong
                if let Some(data) = stitch.llm_request {
                    if let Some(error) = data.get("error").and_then(|v| v.as_str()) {
                        messages.push(Message {
                            role: "assistant".to_string(),
                            content: vec![Content::Text(TextContent {
                                text: format!("[ERROR]: {error}"),
                                cache_control: None,
                            })],
                        });
                    }
                }
            }
        }
    }

    // Add any remaining tool results
    if !pending_tool_results.is_empty() {
        messages.push(Message {
            role: "user".to_string(),
            content: pending_tool_results,
        });
    }

    // Add cache control to the last content of the last message for better performance on subsequent calls
    if let Some(last_message) = messages.last_mut() {
        if let Some(last_content) = last_message.content.last_mut() {
            last_content.set_cache_control(CacheControl {
                r#type: "ephemeral".to_string(),
            });
        }
    }

    Ok(messages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_reconstruct_messages_empty_stitches(pool: PgPool) {
        // Create a thread
        let thread = Thread::create(
            &pool,
            "Test thread".to_string(),
            None,
            None,
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();
        let thread_id = thread.thread_id;

        let messages = reconstruct_messages(&pool, thread_id).await.unwrap();
        assert!(messages.is_empty());
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_reconstruct_messages_with_system_prompt(pool: PgPool) {
        // Create a thread
        let thread = Thread::create(
            &pool,
            "Test thread".to_string(),
            None,
            None,
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();
        let thread_id = thread.thread_id;

        // Create system prompt stitch
        Stitch::create_system_prompt(
            &pool,
            thread_id,
            "You are a helpful AI assistant.".to_string(),
        )
        .await
        .unwrap();

        // Create user message
        Stitch::create_initial_user_message(&pool, thread_id, "Hello")
            .await
            .unwrap();

        let messages = reconstruct_messages(&pool, thread_id).await.unwrap();

        // Should only have user message (system prompts are excluded)
        assert_eq!(messages.len(), 1);

        // Should be user message
        assert_eq!(messages[0].role, "user");
        if let Content::Text(text) = &messages[0].content[0] {
            assert_eq!(text.text, "Hello");
        } else {
            panic!("Expected text content for user message");
        }
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_extract_system_prompt(pool: PgPool) {
        // Create a thread
        let thread = Thread::create(
            &pool,
            "Test thread".to_string(),
            None,
            None,
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();
        let thread_id = thread.thread_id;

        // Test with no system prompt
        let system_prompt = extract_system_prompt(&pool, thread_id).await.unwrap();
        assert!(system_prompt.is_none());

        // Create system prompt stitch
        Stitch::create_system_prompt(
            &pool,
            thread_id,
            "You are a helpful AI assistant.".to_string(),
        )
        .await
        .unwrap();

        // Test with system prompt
        let system_prompt = extract_system_prompt(&pool, thread_id).await.unwrap();
        assert_eq!(
            system_prompt,
            Some("You are a helpful AI assistant.".to_string())
        );
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_extract_system_prompt_multiple_error(pool: PgPool) {
        // Create a thread
        let thread = Thread::create(
            &pool,
            "Test thread".to_string(),
            None,
            None,
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();
        let thread_id = thread.thread_id;

        // Create first system prompt stitch
        Stitch::create_system_prompt(&pool, thread_id, "First system prompt".to_string())
            .await
            .unwrap();

        // Create second system prompt stitch (this should not happen in practice)
        sqlx::query!(
            r#"
            INSERT INTO stitches (thread_id, previous_stitch_id, stitch_type, llm_request)
            VALUES ($1, NULL, 'system_prompt', $2)
            "#,
            thread_id,
            json!({"text": "Second system prompt"})
        )
        .execute(&pool)
        .await
        .unwrap();

        // Should error on multiple system prompts
        let result = extract_system_prompt(&pool, thread_id).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Multiple system prompts found"));
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_reconstruct_messages_single_llm_call(pool: PgPool) {
        // Create a thread
        let thread = Thread::create(
            &pool,
            "Test thread".to_string(),
            None,
            None,
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();
        let thread_id = thread.thread_id;

        // Create initial user message
        let initial_stitch = Stitch::create_initial_user_message(&pool, thread_id, "Hello")
            .await
            .unwrap();

        // Create LLM response
        let response = json!({
            "content": [
                {"type": "text", "text": "Hi there!"}
            ]
        });

        // Create LLM call stitch for the response
        Stitch::create_llm_call(
            &pool,
            thread_id,
            Some(initial_stitch.stitch_id),
            json!({}),
            response,
        )
        .await
        .unwrap();

        let messages = reconstruct_messages(&pool, thread_id).await.unwrap();

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[1].role, "assistant");

        if let Content::Text(text) = &messages[0].content[0] {
            assert_eq!(text.text, "Hello");
        } else {
            panic!("Expected text content");
        }

        if let Content::Text(text) = &messages[1].content[0] {
            assert_eq!(text.text, "Hi there!");
        } else {
            panic!("Expected text content");
        }
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_reconstruct_messages_with_tool_calls(pool: PgPool) {
        // Create a thread
        let thread = Thread::create(
            &pool,
            "Test thread".to_string(),
            None,
            None,
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();
        let thread_id = thread.thread_id;

        // Create initial user message
        let initial_stitch =
            Stitch::create_initial_user_message(&pool, thread_id, "What's the weather?")
                .await
                .unwrap();

        // First LLM response with tool use
        let response1 = json!({
            "content": [
                {"type": "text", "text": "I'll check the weather for you."},
                {
                    "type": "tool_use",
                    "id": "tool_123",
                    "name": "get_weather",
                    "input": {"location": "New York"}
                }
            ]
        });

        let llm_stitch1 = Stitch::create_llm_call(
            &pool,
            thread_id,
            Some(initial_stitch.stitch_id),
            json!({}),
            response1,
        )
        .await
        .unwrap();

        // Tool call
        let tool_stitch = Stitch::create_tool_call(
            &pool,
            thread_id,
            Some(llm_stitch1.stitch_id),
            "get_weather".to_string(),
            json!({"location": "New York"}),
            json!({"temperature": "72F", "condition": "sunny"}),
        )
        .await
        .unwrap();

        // Second LLM call after tool result
        let response2 = json!({
            "content": [
                {"type": "text", "text": "The weather in New York is sunny and 72°F."}
            ]
        });

        Stitch::create_llm_call(
            &pool,
            thread_id,
            Some(tool_stitch.stitch_id),
            json!({}),
            response2,
        )
        .await
        .unwrap();

        let messages = reconstruct_messages(&pool, thread_id).await.unwrap();

        assert_eq!(messages.len(), 4);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[1].role, "assistant");
        assert_eq!(messages[2].role, "user");
        assert_eq!(messages[3].role, "assistant");

        // Check tool use in assistant message
        assert_eq!(messages[1].content.len(), 2);
        if let Content::ToolUse(tool_use) = &messages[1].content[1] {
            assert_eq!(tool_use.name, "get_weather");
            assert_eq!(tool_use.id, "tool_123");
        } else {
            panic!("Expected tool use content");
        }

        // Check tool result in user message
        assert_eq!(messages[2].content.len(), 1);
        if let Content::ToolResult(tool_result) = &messages[2].content[0] {
            assert_eq!(tool_result.tool_use_id, "tool_123");
            assert!(!tool_result.is_error);
            assert!(tool_result.content.contains("72F"));
        } else {
            panic!("Expected tool result content");
        }
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_reconstruct_messages_with_tool_error(pool: PgPool) {
        // Create a thread
        let thread = Thread::create(
            &pool,
            "Test thread".to_string(),
            None,
            None,
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();
        let thread_id = thread.thread_id;

        // Create initial user message
        let initial_stitch = Stitch::create_initial_user_message(&pool, thread_id, "Do something")
            .await
            .unwrap();

        // LLM response with tool use
        let response = json!({
            "content": [
                {
                    "type": "tool_use",
                    "id": "tool_456",
                    "name": "failing_tool",
                    "input": {}
                }
            ]
        });

        let llm_stitch = Stitch::create_llm_call(
            &pool,
            thread_id,
            Some(initial_stitch.stitch_id),
            json!({}),
            response,
        )
        .await
        .unwrap();

        // Tool call with error
        Stitch::create_tool_call(
            &pool,
            thread_id,
            Some(llm_stitch.stitch_id),
            "failing_tool".to_string(),
            json!({}),
            json!({"error": "Tool execution failed"}),
        )
        .await
        .unwrap();

        let messages = reconstruct_messages(&pool, thread_id).await.unwrap();

        assert_eq!(messages.len(), 3);

        // We should have:
        // 1. Initial user message
        // 2. Assistant response with tool use
        // 3. Tool result (added at the end since there's no follow-up LLM call)
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[1].role, "assistant");
        assert_eq!(messages[2].role, "user");

        // Check the tool result has error
        if let Content::ToolResult(tool_result) = &messages[2].content[0] {
            assert_eq!(tool_result.tool_use_id, "tool_456");
            assert!(tool_result.is_error);
            assert_eq!(tool_result.content, "\"Tool execution failed\"");
        } else {
            panic!("Expected tool result content");
        }
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_reconstruct_messages_discord_after_tool_call(pool: PgPool) {
        // Create a thread
        let thread = Thread::create(
            &pool,
            "Test thread".to_string(),
            None,
            None,
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();
        let thread_id = thread.thread_id;

        // Create initial user message
        let initial_stitch = Stitch::create_initial_user_message(&pool, thread_id, "Hello")
            .await
            .unwrap();

        // LLM response with listen tool use
        let response1 = json!({
            "content": [
                {
                    "type": "tool_use",
                    "id": "tool_listen_1",
                    "name": "listen_to_thread",
                    "input": {"duration_seconds": null}
                }
            ]
        });

        let llm_stitch1 = Stitch::create_llm_call(
            &pool,
            thread_id,
            Some(initial_stitch.stitch_id),
            json!({}),
            response1,
        )
        .await
        .unwrap();

        // Tool call result
        let tool_stitch = Stitch::create_tool_call(
            &pool,
            thread_id,
            Some(llm_stitch1.stitch_id),
            "listen_to_thread".to_string(),
            json!({"duration_seconds": null}),
            json!("Listening to thread..."),
        )
        .await
        .unwrap();

        // Discord message arrives after the tool call - create a proper Discord Message struct
        let discord_message_data = json!({
            "id": "1234567890",
            "channel_id": "1111111111",
            "author": {
                "id": "987654321",
                "username": "testuser",
                "discriminator": "1234",
                "global_name": "Test User",
                "avatar": null,
                "bot": false,
                "system": false,
                "mfa_enabled": false,
                "banner": null,
                "accent_color": null,
                "locale": null,
                "verified": null,
                "email": null,
                "flags": 0,
                "premium_type": 0,
                "public_flags": 0,
                "member": null
            },
            "content": "Thanks for listening!",
            "timestamp": "2024-01-01T00:00:00Z",
            "edited_timestamp": null,
            "tts": false,
            "mention_everyone": false,
            "mentions": [],
            "mention_roles": [],
            "mention_channels": [],
            "attachments": [],
            "embeds": [],
            "reactions": [],
            "nonce": null,
            "pinned": false,
            "webhook_id": null,
            "type": 0,
            "activity": null,
            "application": null,
            "application_id": null,
            "flags": 0,
            "message_reference": null,
            "referenced_message": null,
            "interaction_metadata": null,
            "interaction": null,
            "thread": null,
            "components": [],
            "sticker_items": [],
            "stickers": [],
            "position": null,
            "role_subscription_data": null,
            "resolved": null,
            "poll": null,
            "call": null
        });
        let discord_message_data = json!({
            "data": discord_message_data,
            "type": "discord_message",
        });

        Stitch::create_discord_message(
            &pool,
            thread_id,
            Some(tool_stitch.stitch_id),
            discord_message_data,
        )
        .await
        .unwrap();

        let messages = reconstruct_messages(&pool, thread_id).await.unwrap();

        // Should have:
        // 1. Initial user message "Hello"
        // 2. Assistant message with listen tool use
        // 3. User message with tool result
        // 4. User message from Discord
        assert_eq!(messages.len(), 4);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[1].role, "assistant");
        assert_eq!(messages[2].role, "user"); // Tool result
        assert_eq!(messages[3].role, "user"); // Discord message

        // Verify the tool result comes before the Discord message
        if let Content::ToolResult(tool_result) = &messages[2].content[0] {
            assert!(tool_result.content.contains("Listening to thread"));
        } else {
            panic!("Expected tool result in message 2");
        }

        if let Content::Text(text) = &messages[3].content[0] {
            assert_eq!(
                text.text,
                "[Test User (@testuser#1234, ID: 987654321, Message ID: 1234567890)]: Thanks for listening!"
            );
        } else {
            panic!("Expected Discord message text in message 3");
        }
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_reconstruct_messages_multiple_tool_results_grouped(pool: PgPool) {
        // Create a thread
        let thread = Thread::create(
            &pool,
            "Test thread".to_string(),
            None,
            None,
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();
        let thread_id = thread.thread_id;

        // Create initial user message
        let initial_stitch =
            Stitch::create_initial_user_message(&pool, thread_id, "Do multiple things")
                .await
                .unwrap();

        // LLM response with multiple tool uses
        let response = json!({
            "content": [
                {
                    "type": "tool_use",
                    "id": "tool_1",
                    "name": "tool_a",
                    "input": {"param": 1}
                },
                {
                    "type": "tool_use",
                    "id": "tool_2",
                    "name": "tool_b",
                    "input": {"param": 2}
                }
            ]
        });

        let llm_stitch = Stitch::create_llm_call(
            &pool,
            thread_id,
            Some(initial_stitch.stitch_id),
            json!({}),
            response,
        )
        .await
        .unwrap();

        // First tool call
        let tool_stitch1 = Stitch::create_tool_call(
            &pool,
            thread_id,
            Some(llm_stitch.stitch_id),
            "tool_a".to_string(),
            json!({"param": 1}),
            json!({"result": "success_a"}),
        )
        .await
        .unwrap();

        // Second tool call
        Stitch::create_tool_call(
            &pool,
            thread_id,
            Some(tool_stitch1.stitch_id),
            "tool_b".to_string(),
            json!({"param": 2}),
            json!({"result": "success_b"}),
        )
        .await
        .unwrap();

        let messages = reconstruct_messages(&pool, thread_id).await.unwrap();

        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[1].role, "assistant");
        assert_eq!(messages[2].role, "user");

        // Both tool results should be in the same user message
        assert_eq!(messages[2].content.len(), 2);

        if let Content::ToolResult(result1) = &messages[2].content[0] {
            // Check that we either have the exact tool_use_id or a generated one
            assert!(
                result1.tool_use_id == "tool_1" || result1.tool_use_id.starts_with("tool_tool_a_")
            );
            assert!(result1.content.contains("success_a"));
        } else {
            panic!("Expected first tool result");
        }

        if let Content::ToolResult(result2) = &messages[2].content[1] {
            // Check that we either have the exact tool_use_id or a generated one
            assert!(
                result2.tool_use_id == "tool_2" || result2.tool_use_id.starts_with("tool_tool_b_")
            );
            assert!(result2.content.contains("success_b"));
        } else {
            panic!("Expected second tool result");
        }
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_reconstruct_messages_cache_control(pool: PgPool) {
        // Create a thread
        let thread = Thread::create(
            &pool,
            "Test thread".to_string(),
            None,
            None,
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();
        let thread_id = thread.thread_id;

        // Create initial user message
        let initial_stitch = Stitch::create_initial_user_message(&pool, thread_id, "Hello")
            .await
            .unwrap();

        // Create LLM response
        let response = json!({
            "content": [
                {"type": "text", "text": "Hi there!"}
            ]
        });

        Stitch::create_llm_call(
            &pool,
            thread_id,
            Some(initial_stitch.stitch_id),
            json!({}),
            response,
        )
        .await
        .unwrap();

        let messages = reconstruct_messages(&pool, thread_id).await.unwrap();

        // Should have 2 messages
        assert_eq!(messages.len(), 2);

        // First message's last content should not have cache_control
        assert!(messages[0]
            .content
            .last()
            .and_then(|c| c.cache_control())
            .is_none());

        // Last message's last content should have cache_control set to "ephemeral"
        let last_content_cache_control = messages[1].content.last().and_then(|c| c.cache_control());
        assert!(last_content_cache_control.is_some());
        assert_eq!(last_content_cache_control.unwrap().r#type, "ephemeral");

        // Verify JSON serialization
        let json = serde_json::to_value(messages[1].content.last().unwrap()).unwrap();
        assert_eq!(json["cache_control"], json!({"type": "ephemeral"}));

        // Verify that messages without cache_control don't include the field
        let json_without = serde_json::to_value(messages[0].content.last().unwrap()).unwrap();
        assert!(json_without.get("cache_control").is_none());
    }

    #[test]
    fn test_anthropic_request_serialization_with_cache_control() {
        // Create a simple request with cache control on the last message
        let messages = vec![
            Message {
                role: "user".to_string(),
                content: vec![Content::Text(TextContent {
                    text: "Hello".to_string(),
                    cache_control: None,
                })],
            },
            Message {
                role: "assistant".to_string(),
                content: vec![Content::Text(TextContent {
                    text: "Hi there!".to_string(),
                    cache_control: Some(CacheControl {
                        r#type: "ephemeral".to_string(),
                    }),
                })],
            },
        ];

        let request = AnthropicRequest {
            model: "claude-3-opus-20240229".to_string(),
            max_tokens: 1024,
            system: None,
            messages,
            tools: vec![],
            tool_choice: None,
            thinking: None,
        };

        let json = serde_json::to_value(&request).unwrap();

        // Verify the structure
        assert_eq!(json["model"], json!("claude-3-opus-20240229"));
        assert_eq!(json["max_tokens"], json!(1024));

        // Verify the exact JSON structure we expect
        let expected_messages = json!([
            {
                "role": "user",
                "content": [{"type": "text", "text": "Hello"}]
            },
            {
                "role": "assistant",
                "content": [{"type": "text", "text": "Hi there!", "cache_control": {"type": "ephemeral"}}],
            }
        ]);

        assert_eq!(json["messages"], expected_messages);
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_reconstruct_messages_duplicate_tool_calls(pool: PgPool) {
        // Create a thread
        let thread = Thread::create(
            &pool,
            "Test thread".to_string(),
            None,
            None,
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();
        let thread_id = thread.thread_id;

        // Create initial user message
        let initial_stitch = Stitch::create_initial_user_message(&pool, thread_id, "prompt")
            .await
            .unwrap();

        // LLM response with two identical tool uses (same tool name)
        let response = json!({
            "content": [
                {
                    "type": "tool_use",
                    "id": "tool_1",
                    "name": "calculate",
                    "input": {"x": 5, "y": 3}
                },
                {
                    "type": "tool_use",
                    "id": "tool_2",
                    "name": "calculate",
                    "input": {"x": 5, "y": 3}
                }
            ]
        });

        let llm_stitch = Stitch::create_llm_call(
            &pool,
            thread_id,
            Some(initial_stitch.stitch_id),
            json!({}),
            response,
        )
        .await
        .unwrap();

        // First tool call result
        let tool_stitch1 = Stitch::create_tool_call(
            &pool,
            thread_id,
            Some(llm_stitch.stitch_id),
            "calculate".to_string(),
            json!({"x": 5, "y": 3}),
            json!({"result": 8}),
        )
        .await
        .unwrap();

        // Second tool call result (same tool, same params)
        Stitch::create_tool_call(
            &pool,
            thread_id,
            Some(tool_stitch1.stitch_id),
            "calculate".to_string(),
            json!({"x": 5, "y": 3}),
            json!({"result": 8}),
        )
        .await
        .unwrap();

        let messages = reconstruct_messages(&pool, thread_id).await.unwrap();

        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[1].role, "assistant");
        assert_eq!(messages[2].role, "user");

        // Check assistant message has both tool uses
        assert_eq!(messages[1].content.len(), 2);

        // Both tool results should be in the same user message
        assert_eq!(messages[2].content.len(), 2);

        // Check that the first tool result has the correct tool_use_id
        if let Content::ToolResult(result1) = &messages[2].content[0] {
            assert_eq!(
                result1.tool_use_id, "tool_1",
                "First tool result should have tool_use_id 'tool_1'"
            );
            assert!(result1.content.contains('8'));
        } else {
            panic!("Expected first tool result");
        }

        // Check that the second tool result has the correct tool_use_id
        if let Content::ToolResult(result2) = &messages[2].content[1] {
            assert_eq!(
                result2.tool_use_id, "tool_2",
                "Second tool result should have tool_use_id 'tool_2'"
            );
            assert!(result2.content.contains('8'));
        } else {
            panic!("Expected second tool result");
        }
    }
}
