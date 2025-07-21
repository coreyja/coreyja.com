# Discord Interactive Threads Implementation Plan

## Overview

This document outlines how to integrate Discord-based interactive threads into the existing Agentic Threads system. Interactive threads maintain an ongoing conversation with Discord users rather than working toward a specific completion goal.

## Key Design Decisions

### 1. Thread Type Extension

Add a new thread type to the existing system:

```sql
-- Modify the threads table check constraint
CHECK (status IN ('pending', 'running', 'waiting', 'completed', 'failed', 'interactive'))

-- Add thread_type column
ALTER TABLE threads ADD COLUMN thread_type TEXT DEFAULT 'autonomous'
  CHECK (thread_type IN ('autonomous', 'interactive'));

-- Add thread metadata column for Discord-specific data
ALTER TABLE threads ADD COLUMN metadata JSONB DEFAULT '{}'::jsonb;
```

### 2. Discord Metadata Structure

Store Discord-specific information in the thread's metadata field:

```json
{
  "discord": {
    "channel_id": "1234567890",
    "thread_id": "0987654321",
    "guild_id": "5678901234",
    "webhook_url": "https://discord.com/api/webhooks/...",
    "last_message_id": "1111111111",
    "participants": ["user1#1234", "user2#5678"],
    "created_by": "user1#1234",
    "thread_name": "Help with Python debugging"
  }
}
```

### 3. Modified Thread Lifecycle

Interactive threads have a different lifecycle:

- **Creation**: Triggered by Discord slash command or when mentioned in a thread
- **Running**: Continuously processes new messages as they arrive
- **Paused**: Temporarily inactive (e.g., rate limited, user requested)
- **Archived**: Discord thread archived, no new messages expected
- **Never Completes**: Unlike goal-oriented threads, interactive threads don't complete

### 4. New Stitch Types

Add Discord-specific stitch types:

```sql
-- Update stitch_type constraint
CHECK (stitch_type IN ('llm_call', 'tool_call', 'thread_result', 'discord_message', 'discord_action'))
```

- `discord_message`: Incoming message from Discord user
- `discord_action`: Discord API action (send message, add reaction, etc.)

### 5. Discord-Specific Tools

New tools for Discord interactions:

```json
[
  {
    "name": "send_discord_message",
    "description": "Send a message to the Discord thread",
    "parameters": {
      "content": "string",
      "reply_to_message_id": "string (optional)",
      "embeds": "array (optional)"
    }
  },
  {
    "name": "add_discord_reaction",
    "description": "Add a reaction to a Discord message",
    "parameters": {
      "message_id": "string",
      "emoji": "string"
    }
  },
  {
    "name": "get_discord_thread_history",
    "description": "Fetch recent message history from the Discord thread",
    "parameters": {
      "limit": "number (default 50)",
      "before_message_id": "string (optional)"
    }
  },
  {
    "name": "create_discord_thread",
    "description": "Create a new Discord thread",
    "parameters": {
      "name": "string",
      "auto_archive_duration": "number (60, 1440, 4320, 10080)"
    }
  },
  {
    "name": "edit_discord_message",
    "description": "Edit a previously sent message",
    "parameters": {
      "message_id": "string",
      "new_content": "string"
    }
  }
]
```

### 6. Discord Event Processing

Create a new job type for Discord events:

```rust
struct ProcessDiscordEventInput {
    thread_id: Uuid,
    event_type: String, // "message", "reaction", "thread_update"
    event_data: serde_json::Value,
}
```

### 7. Context Management

Interactive threads need special context handling:

1. **Message History Window**: Keep last N messages in context
2. **Participant Tracking**: Track who's in the conversation
3. **Conversation Summary**: Periodically summarize long conversations
4. [For Later] **Topic Detection**: Identify when conversation topic changes significantly

### 8. Integration Architecture

```
Discord Bot/Webhook
      │
      ├─> Discord Event Handler (new service)
      │     │
      │     ├─> Find or Create Interactive Thread
      │     │
      │     └─> Enqueue ProcessDiscordEvent job
      │
      └─> ProcessDiscordEvent (job)
            │
            ├─> Create discord_message Stitch
            │
            ├─> ProcessThreadStep (reuse existing)
            │     │
            │     ├─> LLM processes with Discord context
            │     │
            │     └─> Calls Discord tools
            │
            └─> Updates last_message_id in metadata
```

### 9. Special Behaviors for Interactive Threads

1. **Auto-pause**: After N minutes of inactivity
2. **Rate Limiting**: Prevent spam responses
3. **Mention Detection**: Only respond when mentioned or directly asked
4. **Command Handling**: Special commands like `/pause`, `/summarize`, `/help`
5. **Multi-turn Memory**: Remember context across many messages

### 10. Database Schema Changes

```sql
-- New indexes for Discord queries
CREATE INDEX idx_threads_metadata_discord ON threads ((metadata->'discord'->>'thread_id'))
  WHERE metadata->'discord'->>'thread_id' IS NOT NULL;

-- Track Discord-specific metrics
CREATE TABLE discord_thread_metrics (
    thread_id UUID PRIMARY KEY REFERENCES threads(id),
    message_count INTEGER DEFAULT 0,
    participant_count INTEGER DEFAULT 0,
    last_activity TIMESTAMP WITH TIME ZONE,
    total_tokens_used INTEGER DEFAULT 0
);
```

### 11. System Prompts for Interactive Mode

Special system prompts for Discord context:

```
You are an AI assistant participating in a Discord thread.
- Be conversational and friendly
- Keep responses concise (Discord has a 2000 char limit)
- Use Discord markdown formatting
- Reference users by @mention when appropriate
- Maintain context across the conversation
- You can see message history and reactions
```

### 12. Hybrid Thread Support

Allow threads to be both goal-oriented AND interactive:

- Start as interactive Discord thread
- User asks for specific task → spawn goal-oriented child thread
- Report results back to Discord thread
- Continue conversation

## Implementation Phases

### Phase 1: Foundation

1. Add thread_type and metadata columns
2. Create Discord event handler service
3. Implement basic send_discord_message tool
4. Create ProcessDiscordEvent job

### Phase 2: Core Interaction

1. Implement all Discord tools
2. Add discord_message and discord_action stitch types
3. Handle message history context
4. Implement mention detection

### Phase 3: Advanced Features

1. Conversation summarization
2. Auto-pause and rate limiting
3. Command handling
4. Multi-participant awareness
5. Reaction-based feedback

### Phase 4: Integration

1. Discord bot setup and deployment
2. Webhook configuration
3. Admin UI for Discord thread monitoring
4. Metrics and observability

## Benefits of This Approach

1. **Reuses Existing Infrastructure**: Leverages threads, stitches, and job system
2. **Maintains History**: All Discord interactions stored as stitches
3. **Flexible Context**: Can spawn child threads for complex tasks
4. **Scalable**: Each Discord thread is independent
5. **Debuggable**: Full conversation history in database

## Challenges to Address

1. **Long Conversations**: Need strategies for context window management
2. **Real-time Feel**: Quick response times expected in chat
3. **Multi-user Coordination**: Handling simultaneous messages
4. **Discord API Limits**: Rate limiting and message size constraints
5. **Cost Management**: Long-running threads could use many tokens

## Next Steps

1. Set up Discord application and bot
2. Create database migrations for schema changes
3. Implement Discord event handler service
4. Create first Discord tool (send_message)
5. Test with simple interactive thread
