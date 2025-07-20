use cja::jobs::Job;
use color_eyre::eyre::bail;
use db::agentic_threads::{Stitch, Thread, ThreadStatus};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{types::Uuid, PgPool};

use crate::{
    al::{
        standup::{AnthropicRequest, AnthropicResponse, Content, Message, ToolChoice, ToolResult},
        tools::{
            discord::{CompleteThread, SendDiscordMessage},
            ThreadContext, ToolBag,
        },
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
            | db::agentic_threads::StitchType::ToolCall => {
                // This is the expected types that we can process here right now
            }
        }
    }

    let messages = reconstruct_messages(&app_state.db, thread_id).await?;

    // Set up tools
    let mut tools = ToolBag::default();
    tools.add_tool(SendDiscordMessage::new(app_state.clone()))?;
    tools.add_tool(CompleteThread::new(app_state.clone()))?;
    tools.add_tool(
        crate::al::tools::tool_suggestions::ToolSuggestionsSubmit::new(app_state.clone()),
    )?;

    // Make LLM request
    let request = AnthropicRequest {
        model: "claude-sonnet-4-0".to_string(),
        max_tokens: 1024,
        messages: messages.clone(),
        tools: tools.as_api(),
        tool_choice: Some(ToolChoice {
            r#type: "any".to_string(),
        }),
    };

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &app_state.standup.anthropic_api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(cja::color_eyre::eyre::eyre!(
            "Anthropic API error: {}",
            error_text
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
            Content::ToolUse(tool_use_content) => {
                let tool_name = tool_use_content.name.clone();
                let tool_input = tool_use_content.input.clone();

                // Create thread context with the previous stitch ID
                let context = ThreadContext {
                    thread_id,
                    previous_stitch_id,
                };

                let tool_result = tools.call_tool(tool_use_content, context).await;
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
        }
    }

    Ok(())
}

async fn reconstruct_messages(db: &PgPool, thread_id: Uuid) -> cja::Result<Vec<Message>> {
    let stitches = Stitch::get_by_thread_ordered(db, thread_id).await?;

    let mut messages = Vec::new();
    let mut pending_tool_results: Vec<Content> = Vec::new();

    let mut tool_use_mapping: std::collections::HashMap<(String, Option<Uuid>), String> =
        std::collections::HashMap::new();

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

                if let Some(response) = stitch.llm_response {
                    if let Some(content) = response.get("content") {
                        let content_vec: Vec<Content> = serde_json::from_value(content.clone())?;

                        for content_item in &content_vec {
                            if let Content::ToolUse(tool_use) = content_item {
                                tool_use_mapping.insert(
                                    (tool_use.name.clone(), Some(stitch.stitch_id)),
                                    tool_use.id.clone(),
                                );
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
                    let tool_use_id = tool_use_mapping
                        .get(&(tool_name.clone(), stitch.previous_stitch_id))
                        .cloned()
                        .unwrap_or_else(|| format!("tool_{}_{}", tool_name, stitch.stitch_id));

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
                    }));
                }
            }
            db::agentic_threads::StitchType::ThreadResult => {
                todo!("Thread results would be handled here when we add child thread support");
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

    Ok(messages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_reconstruct_messages_empty_stitches(pool: PgPool) {
        // Create a thread
        let thread = Thread::create(&pool, "Test thread".to_string())
            .await
            .unwrap();
        let thread_id = thread.thread_id;

        let messages = reconstruct_messages(&pool, thread_id).await.unwrap();
        assert!(messages.is_empty());
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_reconstruct_messages_single_llm_call(pool: PgPool) {
        // Create a thread
        let thread = Thread::create(&pool, "Test thread".to_string())
            .await
            .unwrap();
        let thread_id = thread.thread_id;

        // Create initial user message
        let initial_stitch =
            Stitch::create_initial_user_message(&pool, thread_id, "Hello".to_string())
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
        let thread = Thread::create(&pool, "Test thread".to_string())
            .await
            .unwrap();
        let thread_id = thread.thread_id;

        // Create initial user message
        let initial_stitch = Stitch::create_initial_user_message(
            &pool,
            thread_id,
            "What's the weather?".to_string(),
        )
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
        let thread = Thread::create(&pool, "Test thread".to_string())
            .await
            .unwrap();
        let thread_id = thread.thread_id;

        // Create initial user message
        let initial_stitch =
            Stitch::create_initial_user_message(&pool, thread_id, "Do something".to_string())
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
    async fn test_reconstruct_messages_multiple_tool_results_grouped(pool: PgPool) {
        // Create a thread
        let thread = Thread::create(&pool, "Test thread".to_string())
            .await
            .unwrap();
        let thread_id = thread.thread_id;

        // Create initial user message
        let initial_stitch =
            Stitch::create_initial_user_message(&pool, thread_id, "Do multiple things".to_string())
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
}
