# Plan: Fix System Prompt Handling for Anthropic API

## Problem
The current implementation includes system messages in the messages array, which is not allowed by the Anthropic API. System prompts should be provided as a top-level `system` parameter in the request, not as a message with role "system" in the messages array.
There is also a Rust enum. There's just not a database enum. We do want to update the Rust enum, of course.
## Current Implementation Issues

1. **In `thread_processor.rs`**:
   - The `reconstruct_messages` function includes system messages in the messages array
   - The `AnthropicRequest` struct doesn't have a `system` field

2. **In `db/src/agentic_threads/mod.rs`**:
   - `create_system_prompt` stores the system prompt as a message with role "system" in the messages array
   - System prompts are stored as `InitialPrompt` stitch type, which is also used for initial user messages

## Required Changes

### 1. Add new `SystemPrompt` stitch type
- Update the database check constraint to include 'system_prompt' as a valid stitch_type value
- This will cleanly separate system prompts from initial user messages
- Add `SystemPrompt` variant to the `StitchType` enum in Rust code (`db/src/agentic_threads/mod.rs`)

### 2. Update `AnthropicRequest` struct (`server/src/al/standup.rs`)
- Add an optional `system` field to the struct:
  ```rust
  pub struct AnthropicRequest {
      pub model: String,
      pub max_tokens: u32,
      pub system: Option<String>,  // Add this field
      pub messages: Vec<Message>,
      pub tools: Vec<AnthropicTool>,
      pub tool_choice: Option<ToolChoice>,
  }
  ```

### 3. Create new `extract_system_prompt` function (`server/src/jobs/thread_processor.rs`)
- Create a new function that returns `Option<String>`
- Look for a `SystemPrompt` stitch type
- Extract and return the system prompt text from the `llm_request` field (expecting format: `{"text": "system prompt content"}`)
- Error if multiple system prompts are found in a thread

### 4. Update `reconstruct_messages` function (`server/src/jobs/thread_processor.rs`)
- Add a new match arm for `SystemPrompt` stitch type that does nothing (skip it)
- Remove the current logic that handles system messages within `InitialPrompt`
- Only process actual initial user messages from `InitialPrompt` stitches

### 5. Update `process_single_step` function (`server/src/jobs/thread_processor.rs`)
- Call `extract_system_prompt` to get the system prompt
- Pass the system prompt to the `AnthropicRequest` constructor
- Update the match statement for previous stitch types to include `SystemPrompt`

### 6. Update `create_system_prompt` function (`db/src/agentic_threads/mod.rs`)
- Change to use the new `SystemPrompt` stitch type instead of `InitialPrompt`
- Store the system prompt in `llm_request` as: `{"text": "system prompt content"}`
- Update the SQL query to use 'system_prompt' as the stitch_type

### 7. Database Migration
- Create a migration to update the check constraint on the stitch_type column to include 'system_prompt'
- The migration will need to:
  - Drop the existing check constraint
  - Add a new check constraint that includes 'system_prompt' as a valid value

### 8. Update tests
- All tests in `thread_processor.rs` that involve system prompts need to be updated
- `test_reconstruct_messages_with_system_prompt` should verify system messages are excluded from the messages array
- Add new tests for `extract_system_prompt` function
- Update test that creates system prompts to use the new stitch type

## Implementation Order

1. Create database migration to update the stitch_type check constraint
2. Add `SystemPrompt` to the `StitchType` enum in the Rust code
3. Update the `AnthropicRequest` struct to add the `system` field
4. Update `create_system_prompt` to use the new stitch type
5. Create the new `extract_system_prompt` function
6. Update `reconstruct_messages` to handle the new stitch type
7. Update `process_single_step` to use both functions
8. Update all affected tests
9. Run tests to ensure everything works correctly

## Testing Strategy

1. Unit tests should verify:
   - System prompts are extracted correctly
   - Messages array only contains user and assistant messages
   - The API request is formed correctly with system as a top-level field
   - Error is thrown when multiple system prompts exist in a thread

2. Integration testing should verify:
   - The Anthropic API accepts the new request format
   - System prompts are properly applied to the model's behavior
