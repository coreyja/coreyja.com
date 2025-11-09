# Product Roadmap

## Active Roadmap Items

### 1. Agent-Initiated Memory Updates
**Status:** Not Started
**Priority:** High
**Dependencies:** Typed Memory Blocks (✅ Complete)

**Overview:**
Enable the AI agent to update its own memory blocks during conversations, allowing it to learn and remember information about users dynamically without requiring manual admin intervention.

**Key Features:**
- Agent can create/update person memory blocks during Discord conversations
- Agent can record learned preferences, facts, and context about users
- Proper permissions system to limit agent write access to appropriate memory types
- Audit logging for all agent-initiated memory changes
- Prompt instructions guiding the agent on when and how to update memories

**Use Cases:**
- Agent learns user's programming language preferences during conversation
- Agent remembers user's timezone, communication style, or project preferences
- Agent updates person memory: "coreyja prefers Rust over Python for systems programming"
- Next conversation automatically includes these learned preferences in system prompt

**Technical Requirements:**
- Add memory update tool/function to agent's available actions
- Implement permissions system (allow writes to "person" type only, for active conversation users)
- Create audit log table tracking agent-initiated memory changes (who, what, when)
- Add prompt instructions for memory management best practices
- Implement rate limiting to prevent excessive memory updates

**Success Criteria:**
- Agent can successfully update person memory during Discord conversations
- Person memory updates persist and appear in subsequent threads
- Audit logs capture all agent-initiated changes
- Agent follows guidelines about when to update memory (not too frequent, not too rare)
- No ability to modify other memory types (persona, etc.) or other users' memories

**Estimated Complexity:** Medium (2-3 weeks)

---

## Future Roadmap Items

### 8. Advanced Memory System
**Status:** Planning
**Priority:** Medium
**Dependencies:** Typed Memory Blocks (✅ Complete), Agent-Initiated Memory Updates

**Overview:**
Enhance agent memory with semantic search, automatic summarization, and context pruning to maintain relevant long-term memory.

**Key Features:**
- Semantic search across memory blocks using embeddings
- Automatic summarization of long memory blocks
- Context-aware memory retrieval (only inject relevant memories)
- Memory pruning and archiving for old/outdated information
- Memory importance scoring and prioritization

**Success Criteria:**
- Agent can find relevant memories using semantic search
- Long conversations are automatically summarized
- System prompt size stays bounded despite growing memory
- Memory retrieval time remains fast even with thousands of blocks

**Estimated Complexity:** Large (4-6 weeks)

---

## Completed Items

### Typed Memory Blocks
**Completed:** 2025-11-08
**Spec:** `agent-os/specs/typed-memory-blocks/`

Implemented foundational memory structure with type-identifier pairs, two-level CRUD admin UI, and person memory injection for Discord threads.

**Key Achievements:**
- Database schema with flexible (type, identifier) structure
- Admin UI for managing memory blocks at `/admin/memories`
- Person memory injection in Discord threads based on user
- 28 feature-specific tests, all passing
