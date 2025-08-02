# Memory Module Refactoring Plan

## Overview

Consolidate all persona and system prompt creation functionality into a dedicated `memory` module. This will centralize persistent memory concerns and provide a cleaner architecture for managing AI assistant personality and system prompts.

## Current State

- `memory_blocks` table and module exist for storing persona data
- System prompt generation likely scattered across codebase
- No unified interface for managing persistent memory and prompts

## Proposed Structure

```
memory/
├── mod.rs           # Module exports and high-level API
├── blocks.rs        # Memory block storage (move from db/src/memory_blocks.rs)
├── persona.rs       # Persona-specific logic and retrieval
└── prompts.rs       # System prompt generation and templating
```

## Refactoring Steps

### Phase 1: Create Module Structure

1. Create `memory` module directory in appropriate location (likely `src/memory/`)
2. Set up module files with basic structure
3. Create module exports in `mod.rs`

### Phase 2: Migrate Existing Code

1. Move `memory_blocks.rs` functionality into `memory/blocks.rs`
2. Update database module to re-export from new location
3. Find and migrate existing persona-related code to `memory/persona.rs`
4. Locate system prompt generation code and consolidate in `memory/prompts.rs`

### Phase 3: Build Unified API

1. Create `MemoryManager` struct as main interface
2. Implement methods:
   - `get_persona()` - Retrieve current persona configuration
   - `update_persona()` - Update persona content
   - `generate_system_prompt()` - Build complete system prompt with persona

### Phase 4: Integration Points

1. Identify all locations using persona/prompt generation
2. Update to use new `MemoryManager` API
3. Ensure backward compatibility during migration

## Key Considerations

### Database Layer

- Keep database queries in the memory module
- Consider whether to keep sqlx queries or abstract further
- Maintain existing migration compatibility

### Extensibility

- Design for future memory block types beyond "persona"
- Allow for different prompt generation strategies
- Support for additional persistent memory types (facts, preferences, etc.)

### Testing

- Unit tests for each module component
- Integration tests for full memory management flow
- Test migration path to ensure no breakage
