# AI Agent Modes Implementation Plan

## Overview

This plan outlines how to add different modes to the AI agent, starting with a cooking mode as the first implementation. The system will allow Discord channels to be associated with specific modes, automatically selecting the appropriate mode based on the channel context.

## Current Architecture Analysis

### Existing Components

1. **Thread System**
   - `Thread` struct with `goal`, `status`, `thread_type` (Autonomous/Interactive)
   - `Stitch` system for tracking individual steps/messages
   - `ThreadBuilder` for creating new threads with Discord metadata

2. **Discord Integration**
   - `DiscordThreadMetadata` links threads to Discord channels/threads
   - Stores `channel_id`, `guild_id`, `thread_name`
   - Discord messages processor creates interactive threads

3. **Memory System**
   - `MemoryBlock` table with `block_type` (currently only 'persona')
   - `MemoryManager` handles memory operations
   - System prompts generated dynamically with persona integration

4. **System Prompts**
   - `PromptGenerator` creates system prompts based on thread type
   - Includes base instructions, persona, and Discord-specific instructions
   - Different behavior for Interactive vs Autonomous threads

## Proposed Mode System Design

### Core Concepts

1. **Mode Definition**
   - Each mode represents a specialized behavior set (cooking, coding, general, etc.)
   - Modes define: system prompts, available tools, memory contexts, behavioral guidelines

2. **Channel-Mode Mapping**
   - Discord channels linked to specific modes
   - When thread created in channel, inherits channel's mode
   - Default mode for unmapped channels

### Database Schema Changes

```sql
-- 1. Add mode to threads table
ALTER TABLE threads ADD COLUMN mode text;
ALTER TABLE threads ADD CONSTRAINT threads_mode_check 
    CHECK (mode IN ('general', 'cooking', 'project_manager'));
UPDATE threads SET mode = 'general' WHERE mode IS NULL;
ALTER TABLE threads ALTER COLUMN mode SET NOT NULL;
ALTER TABLE threads ALTER COLUMN mode SET DEFAULT 'general';

-- 2. Add mode column to existing DiscordChannels table
ALTER TABLE DiscordChannels ADD COLUMN mode text;
ALTER TABLE DiscordChannels ADD CONSTRAINT discord_channels_mode_check 
    CHECK (mode IN ('general', 'cooking', 'project_manager'));
-- Default all existing channels to 'general' mode
UPDATE DiscordChannels SET mode = 'general' WHERE mode IS NULL;
ALTER TABLE DiscordChannels ALTER COLUMN mode SET DEFAULT 'general';
```

### Code Structure Changes

#### 1. Create Mode Enum
```rust
// db/src/agentic_threads/mod.rs
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "text")]
pub enum ThreadMode {
    General,
    Cooking,
    ProjectManager,
}
```

#### 2. Update Thread Model
```rust
// db/src/agentic_threads/mod.rs
pub struct Thread {
    // ... existing fields
    pub mode: ThreadMode, // Defaults to General
}
```

#### 3. Update DiscordChannel Model
```rust
// db/src/lib.rs (or wherever DiscordChannel is defined)
pub struct DiscordChannel {
    // ... existing fields
    pub mode: ThreadMode, // Defaults to General
}
```

#### 4. Mode-Specific System Prompts
```rust
// server/src/memory/prompts.rs
impl PromptGenerator {
    pub async fn generate_mode_instructions(mode: &ThreadMode) -> &'static str {
        match mode {
            ThreadMode::Cooking => {
                "You are a culinary assistant specializing in:
                - Recipe suggestions and modifications
                - Meal planning and preparation
                - Ingredient substitutions
                - Cooking techniques and tips
                - Dietary accommodations
                Track recipes discussed and modifications made."
            },
            ThreadMode::ProjectManager => {
                "You are a project management assistant specializing in:
                - Task breakdown and prioritization
                - Timeline and milestone planning
                - Resource allocation and dependency tracking
                - Risk assessment and mitigation strategies
                - Progress monitoring and status reporting
                Help organize work into actionable tasks with clear deliverables."
            },
            ThreadMode::General => {
                "" // General mode uses standard base instructions
            }
        }
    }
}
```

#### 5. Mode-Specific Tools
```rust
// server/src/al/tools/mod.rs
pub fn get_tools_for_mode(mode: &ThreadMode) -> Vec<Tool> {
    let mut tools = vec![/* base tools always available */];
    
    match mode {
        ThreadMode::Cooking => {
            tools.extend(vec![
                Tool::SearchRecipes,
                Tool::SaveRecipe,
                Tool::GetRecipeHistory,
                Tool::UpdateInventory,
                // Future: ScanReceipt, PlanMeals
            ]);
        },
        ThreadMode::ProjectManager => {
            tools.extend(vec![
                Tool::CreateTask,
                Tool::UpdateTaskStatus,
                Tool::CreateMilestone,
                Tool::TrackDependencies,
                Tool::GenerateStatusReport,
                // Future: GanttChart, ResourcePlanning
            ]);
        },
        ThreadMode::General => {
            // Standard tool set, no mode-specific additions
        }
    }
    tools
}
```

### Implementation Phases

## Phase 1: Core Mode Infrastructure (Week 1)

1. **Database Migration**
   - Add mode column to threads table
   - Add mode column to DiscordChannels table
   - Set default mode for existing channels

2. **Model Updates**
   - Add ThreadMode enum
   - Update Thread struct
   - Update DiscordChannel struct with mode field

3. **Thread Creation Updates**
   - ThreadBuilder accepts mode parameter
   - Look up channel mode from DiscordChannels when creating Discord threads
   - Default to 'general' mode if not specified

4. **System Prompt Integration**
   - Update PromptGenerator for mode-specific instructions
   - Combine base + persona + mode + context instructions

## Phase 2: Cooking Mode MVP (Week 2)

1. **Cooking-Specific System Prompt**
   - Culinary expertise instructions
   - Recipe tracking guidelines
   - Meal planning context

2. **Basic Recipe Tools**
   - SearchRecipes: Find recipes from web/database
   - SaveRecipe: Store recipes with metadata
   - GetRecipeHistory: Retrieve past recipes and modifications

3. **Recipe Storage**
   - Create recipes table
   - Store full recipe content (not just links)
   - Track modifications and ratings

4. **Discord Channel Setup**
   - Create cooking channel
   - Map channel to cooking mode
   - Test thread creation with cooking mode

## Phase 3: Project Manager Mode (Week 3)

1. **Project Management Tools**
   - Task creation and tracking
   - Milestone management
   - Dependency tracking between tasks
   - Status report generation

2. **Project Context**
   - Store project goals and constraints
   - Track team members and resources
   - Timeline and deadline management

3. **Integration Features**
   - Link tasks to Discord threads
   - Progress notifications
   - Weekly status summaries

## Phase 4: Enhanced Cooking Features (Week 4)

1. **Recipe Management**
   - Recipe rating system
   - Post-meal feedback collection
   - Recipe version history

2. **Cooking Context Tracking**
   - Store cooking preferences per thread
   - Track dietary restrictions in thread metadata
   - Remember favorite recipes in recipe history

3. **Proactive Interactions**
   - Meal planning reminders
   - Recipe suggestions based on history
   - Follow-up on recent meals

## Phase 5: Mode Management UI (Week 5)

1. **Admin Interface**
   - View/edit DiscordChannels mode settings
   - Create new modes
   - View mode usage statistics

2. **Mode Switching**
   - Command to change channel mode
   - Temporary mode override for threads
   - Mode inheritance for child threads

## Future Enhancements

### Additional Modes
- **Writing Mode**: Blog posts, documentation, creative writing
- **Learning Mode**: Educational content, study assistance
- **Coding Mode**: Code review, debugging, architecture discussions

### Advanced Features
- Mid-thread mode switching
- Mode-specific context persistence
- Cross-mode knowledge sharing
- Mode performance analytics

### Cooking Mode Phase 2
- Inventory management system
- Receipt scanning and processing
- Instacart integration
- Automated shopping lists
- Meal planning calendar

## Testing Strategy

1. **Unit Tests**
   - Mode selection logic
   - Tool filtering by mode
   - System prompt generation

2. **Integration Tests**
   - Thread creation with modes
   - Channel mode lookup from DiscordChannels
   - Mode-specific tool execution

3. **E2E Tests**
   - Discord message to thread creation
   - Mode inheritance
   - Tool availability per mode

## Migration Path

1. All existing threads get 'general' mode (preserving current behavior)
2. All channels default to 'general' mode initially
3. Gradual rollout: change specific channels to cooking/project_manager modes as needed
4. Monitor and adjust based on usage

## Success Metrics

- Successful mode selection based on channel
- Mode-appropriate responses
- Tool usage aligned with mode
- User satisfaction with specialized behavior
- Reduced context switching for mode-specific tasks

## Risk Mitigation

1. **Backwards Compatibility**
   - General mode preserves existing functionality
   - Gradual rollout minimizes disruption

2. **Mode Confusion**
   - Clear mode indicators in responses
   - Ability to query current mode
   - Override mechanisms available

3. **Performance**
   - Lazy load mode-specific tools
   - Cache mode mappings
   - Optimize prompt generation

## Development Checklist

- [ ] Database migrations for threads and DiscordChannels
- [ ] ThreadMode enum implementation (General, Cooking, ProjectManager)
- [ ] Update DiscordChannel model with mode field
- [ ] ThreadBuilder mode support
- [ ] Mode lookup from DiscordChannels on thread creation
- [ ] Mode-specific system prompts
- [ ] Mode-specific tool filtering
- [ ] Cooking mode system prompt
- [ ] Basic recipe tools
- [ ] Recipe storage schema
- [ ] Project Manager mode system prompt
- [ ] Basic project management tools
- [ ] Task/milestone storage schema
- [ ] Discord commands to set channel mode
- [ ] Mode indicator in thread viewer
- [ ] Tests for mode selection
- [ ] Tests for cooking mode
- [ ] Tests for project manager mode
- [ ] Documentation updates