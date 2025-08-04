# Interactive Thread Integration Architecture

## Overview

This document explores integration options for combining Linear Agents with the existing Interactive Discord Thread model. The goal is to create a unified system that can handle interactions from both Discord and Linear while maintaining consistency and avoiding duplication.

## Executive Summary

**Recommended Approach:** Platform-specific attachment tables that link to a shared thread model.

- Keep existing `discord_thread_metadata` table unchanged
- Create new `linear_thread_metadata` table with Linear-specific fields
- Both tables reference the same `thread_id`, allowing cross-platform conversations
- Provides type safety, no migration needed, and natural cross-platform support

## Current State

### Interactive Threads

- Generic thread system with two types: `Autonomous` and `Interactive`
- Threads track conversation history through "stitches" (individual interactions)
- Each thread has a status: `pending`, `running`, `waiting`, `completed`, `failed`, `aborted`
- Interactive threads can be linked to Discord via `discord_thread_metadata` table

### Linear Agents (Planned)

- Session-based model with states: `pending`, `active`, `error`, `awaitingInput`, `complete`
- Communicate through activities: `thought`, `elicitation`, `action`, `response`, `error`
- Respond to mentions and issue assignments
- 30-minute session timeout

## Architecture: Platform-Specific Attachment Tables

The chosen approach uses **dedicated tables for each platform** while maintaining the flexibility of multiple attachments:

1. **Type Safety**: Each platform gets properly typed columns instead of JSONB
2. **Natural Cross-Platform Support**: A single thread can have attachments to multiple platforms
3. **Clean Architecture**: Keeps the core thread model simple and extends via platform-specific tables
4. **No Migration Required**: Keep existing `discord_thread_metadata` table as-is
5. **Better Constraints**: Platform-specific validation and foreign keys

### Implementation Plan

1. **Keep Existing Discord Table**

```sql
-- No changes needed to discord_thread_metadata
-- It already links threads to Discord via thread_id
```

2. **Create Linear Thread Metadata Table**

```sql
CREATE TABLE linear_thread_metadata (
    thread_id UUID PRIMARY KEY REFERENCES threads(thread_id),
    session_id VARCHAR(255) NOT NULL UNIQUE,
    workspace_id VARCHAR(255) NOT NULL,
    issue_id VARCHAR(255),
    issue_title TEXT,
    project_id VARCHAR(255),
    team_id VARCHAR(255),
    created_by_user_id VARCHAR(255) NOT NULL,
    session_status TEXT NOT NULL DEFAULT 'pending',
    last_activity_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    CONSTRAINT check_session_status CHECK (
        session_status IN ('pending', 'active', 'error', 'awaitingInput', 'complete')
    )
);

CREATE INDEX idx_linear_thread_metadata_session_id ON linear_thread_metadata(session_id);
CREATE INDEX idx_linear_thread_metadata_issue_id ON linear_thread_metadata(issue_id);
CREATE INDEX idx_linear_thread_metadata_workspace_id ON linear_thread_metadata(workspace_id);
```

3. **Update Stitch Types**

```sql
-- Drop the existing check constraint
ALTER TABLE stitches DROP CONSTRAINT IF EXISTS stitches_stitch_type_check;

-- Add the new constraint with additional stitch types
ALTER TABLE stitches ADD CONSTRAINT stitches_stitch_type_check 
CHECK (stitch_type IN (
    'initial_prompt',
    'llm_call',
    'tool_call',
    'thread_result',
    'discord_message',
    'agent_thought',          -- NEW: Internal agent reasoning
    'clarification_request',  -- NEW: Requesting user clarification  
    'error'                   -- NEW: Error states
));
```

4. **Create Platform Metadata Models**

```rust
// In db/src/linear_threads.rs
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LinearThreadMetadata {
    pub thread_id: Uuid,
    pub session_id: String,
    pub workspace_id: String,
    pub issue_id: Option<String>,
    pub issue_title: Option<String>,
    pub project_id: Option<String>,
    pub team_id: Option<String>,
    pub created_by_user_id: String,
    pub session_status: String,
    pub last_activity_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl LinearThreadMetadata {
    pub async fn create(pool: &PgPool, /* params */) -> Result<Self> { }
    pub async fn find_by_session_id(pool: &PgPool, session_id: &str) -> Result<Option<Self>> { }
    pub async fn find_by_thread_id(pool: &PgPool, thread_id: Uuid) -> Result<Option<Self>> { }
    pub async fn update_session_status(pool: &PgPool, thread_id: Uuid, status: &str) -> Result<Option<Self>> { }
}
```

5. **Extend ThreadBuilder for Linear**

```rust
// In server/src/agentic_threads/builder.rs

pub struct LinearMetadata {
    pub session_id: String,
    pub workspace_id: String,
    pub issue_id: Option<String>,
    pub issue_title: Option<String>,
    pub project_id: Option<String>,
    pub team_id: Option<String>,
    pub created_by_user_id: String,
}

impl ThreadBuilder {
    // Add Linear metadata field
    linear_metadata: Option<LinearMetadata>,

    pub fn interactive_linear(mut self, metadata: LinearMetadata) -> Self {
        self.thread_type = ThreadType::Interactive;
        self.linear_metadata = Some(metadata);
        self
    }

    // Update build() to handle Linear metadata
    pub async fn build(self) -> Result<Thread> {
        // ... existing validation ...

        // Create Linear metadata if this is a Linear interactive thread
        if let Some(linear_meta) = self.linear_metadata {
            LinearThreadMetadata::create(
                &self.pool,
                thread.thread_id,
                linear_meta.session_id,
                linear_meta.workspace_id,
                // ... other fields
            ).await?;
        }

        // ... rest of build logic ...
    }
}
```

6. **Thread Extension Methods**

```rust
impl Thread {
    pub async fn get_discord_metadata(&self, pool: &PgPool) -> Result<Option<DiscordThreadMetadata>> {
        DiscordThreadMetadata::find_by_thread_id(pool, self.thread_id).await
    }

    pub async fn get_linear_metadata(&self, pool: &PgPool) -> Result<Option<LinearThreadMetadata>> {
        LinearThreadMetadata::find_by_thread_id(pool, self.thread_id).await
    }
}
```

### Usage Examples

The codebase now uses a `ThreadBuilder` pattern for thread creation, which handles:

- Thread creation with proper validation
- Automatic system prompt generation via `MemoryManager`
- Platform metadata creation in a single transaction

**Discord-only thread (current implementation):**

```rust
use crate::agentic_threads::{ThreadBuilder, DiscordMetadata};

let discord_metadata = DiscordMetadata {
    discord_thread_id: thread.id.to_string(),
    channel_id: channel_id.to_string(),
    guild_id: guild_id.to_string(),
    created_by: msg.author.tag(),
    thread_name: thread_name.clone(),
};

let thread = ThreadBuilder::new(pool.clone())
    .with_goal(format!("Interactive Discord thread: {thread_name}"))
    .interactive_discord(discord_metadata)
    .build()
    .await?;
```

**Linear-only thread (proposed):**

```rust
// Extend ThreadBuilder with Linear support
impl ThreadBuilder {
    pub fn interactive_linear(mut self, metadata: LinearMetadata) -> Self {
        self.thread_type = ThreadType::Interactive;
        self.linear_metadata = Some(metadata);
        self
    }
}

// Usage
let linear_metadata = LinearMetadata {
    session_id: webhook.session_id,
    workspace_id: webhook.workspace_id,
    issue_id: webhook.issue_id,
    // ... other fields
};

let thread = ThreadBuilder::new(pool.clone())
    .with_goal(format!("Linear issue: {}", issue_title))
    .interactive_linear(linear_metadata)
    .build()
    .await?;
```

**Cross-platform thread (Linear → Discord):**

```rust
// Initial Linear thread creation
let thread = ThreadBuilder::new(pool.clone())
    .with_goal("Research API options")
    .interactive_linear(linear_metadata)
    .build()
    .await?;

// Later, when user continues in Discord
// Simply create Discord metadata for the existing thread
let discord_meta = DiscordThreadMetadata::create(
    pool,
    thread.thread_id, // Link to existing thread!
    discord_thread_id,
    channel_id,
    guild_id,
    created_by,
    thread_name,
).await?;
```

**Autonomous thread (for background tasks):**

```rust
let thread = ThreadBuilder::new(pool.clone())
    .with_goal("Generate daily standup message")
    .autonomous()
    .build()
    .await?;
```

### Benefits of This Approach

1. **Type Safety**: Each platform has properly typed fields with constraints
2. **No Migration**: Existing Discord threads work without changes
3. **Unified History**: All interactions stored as stitches regardless of platform
4. **Cross-Platform**: Threads can naturally span multiple platforms
5. **Clean Queries**: No JSONB parsing needed, direct column access
6. **Platform Independence**: Core thread logic doesn't need to know about platforms

### New Generic Stitch Types

To support Linear and future platforms, we need to add new generic stitch types:

#### 1. **agent_thought** - For internal agent reasoning

This stitch type captures the agent's internal thinking process, useful for debugging and transparency:

```rust
impl Stitch {
    pub async fn create_agent_thought(
        pool: &PgPool,
        thread_id: Uuid,
        previous_stitch_id: Option<Uuid>,
        thought: String,
        metadata: Option<JsonValue>,
    ) -> Result<Self> {
        let thought_data = json!({
            "thought": thought,
            "metadata": metadata,
            "timestamp": Utc::now().to_rfc3339(),
        });
        
        let stitch = sqlx::query_as!(
            Stitch,
            r#"
            INSERT INTO stitches (thread_id, previous_stitch_id, stitch_type, llm_request)
            VALUES ($1, $2, 'agent_thought', $3)
            RETURNING *
            "#,
            thread_id,
            previous_stitch_id,
            thought_data
        )
        .fetch_one(pool)
        .await?;
        
        Ok(stitch)
    }
}
```

#### 2. **clarification_request** - For requesting clarification from users

When the agent needs more information from the user:

```rust
impl Stitch {
    pub async fn create_clarification_request(
        pool: &PgPool,
        thread_id: Uuid,
        previous_stitch_id: Option<Uuid>,
        question: String,
        context: Option<JsonValue>,
    ) -> Result<Self> {
        let request_data = json!({
            "question": question,
            "context": context,
            "awaiting_response": true,
            "timestamp": Utc::now().to_rfc3339(),
        });
        
        let stitch = sqlx::query_as!(
            Stitch,
            r#"
            INSERT INTO stitches (thread_id, previous_stitch_id, stitch_type, llm_request)
            VALUES ($1, $2, 'clarification_request', $3)
            RETURNING *
            "#,
            thread_id,
            previous_stitch_id,
            request_data
        )
        .fetch_one(pool)
        .await?;
        
        Ok(stitch)
    }
}
```

#### 3. **error** - For error states

A generic error stitch type for any platform:

```rust
impl Stitch {
    pub async fn create_error(
        pool: &PgPool,
        thread_id: Uuid,
        previous_stitch_id: Option<Uuid>,
        error_message: String,
        error_details: Option<JsonValue>,
    ) -> Result<Self> {
        let error_data = json!({
            "error_message": error_message,
            "error_details": error_details,
            "timestamp": Utc::now().to_rfc3339(),
        });
        
        let stitch = sqlx::query_as!(
            Stitch,
            r#"
            INSERT INTO stitches (thread_id, previous_stitch_id, stitch_type, llm_request)
            VALUES ($1, $2, 'error', $3)
            RETURNING *
            "#,
            thread_id,
            previous_stitch_id,
            error_data
        )
        .fetch_one(pool)
        .await?;
        
        Ok(stitch)
    }
}
```

### Handling Platform-Specific Features

**Linear Activities → Generic Stitch Types:**

```rust
// When receiving Linear webhook with activity
match activity_type {
    "thought" => {
        // Map to generic agent_thought type
        Stitch::create_agent_thought(
            pool, 
            thread_id, 
            prev_stitch_id,
            activity.content,
            Some(json!({
                "source": "linear",
                "session_id": session_id
            }))
        ).await?
    },
    "action" => {
        // Map to existing tool_call stitch
        Stitch::create_tool_call(
            pool, 
            thread_id, 
            prev_stitch_id,
            activity.tool_name,
            activity.tool_input,
            activity.tool_output
        ).await?
    },
    "response" => {
        // Map to existing llm_call stitch 
        Stitch::create_llm_call(
            pool, 
            thread_id, 
            prev_stitch_id,
            json!({"role": "assistant"}),
            json!({"content": activity.content})
        ).await?
    },
    "elicitation" => {
        // Map to generic clarification_request type
        Stitch::create_clarification_request(
            pool, 
            thread_id, 
            prev_stitch_id,
            activity.question,
            Some(json!({
                "source": "linear",
                "session_id": session_id
            }))
        ).await?
    },
    "error" => {
        // Map to generic error type
        Stitch::create_error(
            pool, 
            thread_id, 
            prev_stitch_id,
            activity.error_message,
            Some(json!({
                "source": "linear",
                "session_id": session_id,
                "details": activity.error_details
            }))
        ).await?
    }
}
```

**Linear User Prompts:**
```rust
// When user mentions agent or delegates issue
// Use existing initial_prompt stitch type with Linear context
Stitch::create_initial_user_message(
    pool,
    thread_id,
    format!("{}\n\nContext: Linear issue #{} - {}", 
        prompt_text, 
        issue_id, 
        issue_title
    )
).await?
```

**Why This Approach:**
- **Platform-agnostic types** that any integration can use
- **Reuse existing types** where possible (initial_prompt, tool_call, llm_call)
- **Generic new types** for common patterns (thoughts, elicitations, errors)
- **Platform context** stored in metadata, not in type names
- **Future-proof** for adding more platforms

**Discord Messages → Stitches:**

- Continue using `discord_message` stitch type as-is
- No changes needed to existing Discord handling

### Practical Considerations

**Finding Threads Across Platforms:**

```rust
// Find thread by Linear session
pub async fn find_thread_by_linear_session(pool: &PgPool, session_id: &str) -> Result<Option<Thread>> {
    if let Some(linear_meta) = LinearThreadMetadata::find_by_session_id(pool, session_id).await? {
        Thread::get_by_id(pool, linear_meta.thread_id).await
    } else {
        Ok(None)
    }
}

// Find thread by Discord thread ID
pub async fn find_thread_by_discord_id(pool: &PgPool, discord_id: &str) -> Result<Option<Thread>> {
    if let Some(discord_meta) = DiscordThreadMetadata::find_by_discord_thread_id(pool, discord_id).await? {
        Thread::get_by_id(pool, discord_meta.thread_id).await
    } else {
        Ok(None)
    }
}
```

**Handling Platform Disconnections:**

```sql
-- Soft delete for platform metadata (useful if Discord thread is archived or Linear session expires)
ALTER TABLE discord_thread_metadata ADD COLUMN is_active BOOLEAN DEFAULT true;
ALTER TABLE linear_thread_metadata ADD COLUMN is_active BOOLEAN DEFAULT true;
```

**Platform-Specific Views:**

```sql
-- View for all active Linear threads
CREATE VIEW active_linear_threads AS
SELECT
    t.*,
    l.session_id,
    l.issue_id,
    l.session_status
FROM threads t
JOIN linear_thread_metadata l ON t.thread_id = l.thread_id
WHERE l.is_active = true AND t.thread_type = 'interactive';

-- View for cross-platform threads
CREATE VIEW cross_platform_threads AS
SELECT
    t.thread_id,
    t.goal,
    t.status,
    EXISTS(SELECT 1 FROM discord_thread_metadata d WHERE d.thread_id = t.thread_id) as has_discord,
    EXISTS(SELECT 1 FROM linear_thread_metadata l WHERE l.thread_id = t.thread_id) as has_linear
FROM threads t
WHERE t.thread_type = 'interactive';
```

### Next Steps

1. Create the `linear_thread_metadata` table
2. Add `linear_activity` to stitch_type enum
3. Implement Linear webhook handlers that create/update metadata
4. Update thread processor to handle both Discord and Linear sources
5. Create helper functions for cross-platform thread discovery
6. Add platform-aware routing in the thread processor
