# Refactor Plan: Thread Builder Pattern with Integrated System Prompt

## Overview

Implement a builder pattern for thread creation that automatically generates and stores system prompts, ensuring threads are always created with proper initialization.

## Current State

- Thread creation and system prompt creation are two separate steps
- Each location that creates threads must remember to also create the system prompt
- This is error-prone and violates DRY principles

## Proposed Solution: Thread Builder Pattern

### 1. Create ThreadBuilder

Add to `server/src/agentic_threads/builder.rs`:

```rust
use db::agentic_threads::{Thread, ThreadType, Stitch};
use sqlx::PgPool;
use uuid::Uuid;
use crate::memory::MemoryManager;

pub struct ThreadBuilder {
    pool: PgPool,
    goal: String,
    thread_type: ThreadType,
    branching_stitch_id: Option<Uuid>,
}

impl ThreadBuilder {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            goal: String::new(),
            thread_type: ThreadType::Autonomous,
            branching_stitch_id: None,
            custom_system_prompt: None,
        }
    }

    pub fn with_goal(mut self, goal: impl Into<String>) -> Self {
        self.goal = goal.into();
        self
    }

    pub fn with_thread_type(mut self, thread_type: ThreadType) -> Self {
        self.thread_type = thread_type;
        self
    }

    pub fn as_child_of(mut self, parent_stitch_id: Uuid) -> Self {
        self.branching_stitch_id = Some(parent_stitch_id);
        self
    }

    pub async fn build(self) -> Result<Thread> {
        // Validate
        if self.goal.is_empty() {
            return Err(eyre!("Thread goal cannot be empty"));
        }

        // Create the thread using the appropriate method
        let thread = match (self.thread_type, self.branching_stitch_id) {
            (_, Some(parent_id)) => {
                Thread::create_child(&self.pool, parent_id, self.goal).await?
            }
            (ThreadType::Interactive, None) => {
                Thread::create_interactive(&self.pool, self.goal).await?
            }
            (ThreadType::Autonomous, None) => {
                Thread::create(&self.pool, self.goal).await?
            }
        };

        let memory_manager = MemoryManager::new(self.pool.clone());
        let is_discord = self.thread_type == ThreadType::Interactive;
        let system_prompt =memory_manager.generate_system_prompt(is_discord).await?;

        // Create system prompt stitch
        Stitch::create_system_prompt(&self.pool, thread.thread_id, system_prompt).await?;

        Ok(thread)
    }
}
```

### 2. Update All Usage Sites

#### HTTP API (`server/src/http_server/api/threads.rs`)

```rust
let thread = ThreadBuilder::new(state.db().clone())
    .with_goal(payload.goal)
    .with_thread_type(ThreadType::Autonomous)
    .build()
    .await
    .context("Failed to create thread")
    .with_status(StatusCode::INTERNAL_SERVER_ERROR)?;
```

#### Discord (`server/src/jobs/discord_message_processor.rs`)

```rust
let thread = ThreadBuilder::new(db.clone())
    .with_goal(format!("Interactive Discord thread: {thread_name}"))
    .with_thread_type(ThreadType::Interactive)
    .build()
    .await?;
```

#### Standup (`server/src/al/standup.rs`)

```rust
let thread = ThreadBuilder::new(self.app_state.db.clone())
    .with_goal("Generate daily standup message")
    .with_thread_type(ThreadType::Autonomous)
    .build()
    .await?;
```

### 5. Update Tests

Tests should also use the builder for integration tests
Or test the db methods directly.

Example test helper:

```rust
impl ThreadBuilder {
    #[cfg(test)]
    pub fn without_system_prompt(mut self) -> Self {
        self.custom_system_prompt = Some(String::new());
        self
    }
}
```

## Benefits

1. **Flexibility**: Easy to add new configuration options
2. **Readability**: Clear, self-documenting API
3. **Type Safety**: Compile-time validation of required fields
4. **Extensibility**: Easy to add features like custom prompts, metadata, etc.
5. **DRY**: System prompt logic centralized in one place
6. **Testability**: Can easily create threads with specific configurations

## Migration Steps

1. Create the ThreadBuilder in `server/src/agentic_threads/builder.rs`
2. Add module declaration in `server/src/agentic_threads/mod.rs`
3. Update each usage site to use the builder
4. Remove manual system prompt creation from all sites
5. Update tests to use builder or db methods as appropriate
6. Run all tests to ensure nothing breaks

## Future Enhancements

The builder pattern makes it easy to add:

- Custom personas per thread
- Thread metadata
- Different prompt templates
- Thread priorities
- Execution constraints

## Testing Strategy

1. Unit tests for the builder itself
2. Integration tests verifying threads are created with system prompts
3. Test builder validation (e.g., empty goal)
4. Test all thread type combinations
5. Test custom system prompt functionality
