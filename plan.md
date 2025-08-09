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
                // Recipe Management
                Tool::SaveRecipe,
                Tool::SearchRecipes,
                Tool::GetRecipeHistory,
                Tool::GetRecipeStats,
                Tool::AnalyzeRecipeNutrition,
                
                // Meal Tracking
                Tool::RecordMeal,      // Track meal preparation with modifications
                Tool::RateMeal,        // Add rating and comments to meals
                Tool::GetMealHistory,  // View past meals with ratings
                
                // Tag Management
                Tool::CreateTag,
                Tool::AddTagToRecipe,
                Tool::ListTags,
                
                // Location Management
                Tool::CreateLocation,
                Tool::ListLocations,
                Tool::MoveInventory,
                
                // Unit Management
                Tool::CreateUnit,
                Tool::ListUnits,
                
                // Inventory Management
                Tool::UpdateInventory,
                Tool::CheckInventory,
                Tool::SearchInventory,
                Tool::GetExpiringItems,
                
                // Meal Planning
                Tool::CreateMealPlan,
                Tool::AddToMealPlan,
                Tool::GetMealPlan,
                Tool::GetUpcomingMeals,
                Tool::GetMealPlanShopping,
                
                // Smart Features
                Tool::SuggestRecipesByInventory,
                
                // Future: ScanReceipt, ImportRecipeFromURL
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
   - Meal execution and feedback tracking

2. **Basic Recipe Tools**

   - SearchRecipes: Find recipes from web/database
   - SaveRecipe: Store recipes with metadata
   - GetRecipeHistory: Retrieve past recipes and modifications
   - **RecordMeal**: Track when a meal is made with modifications
   - **RateMeal**: Add rating and comments to completed meals

3. **Recipe & Meal Storage**

   - Create recipes table
   - Store full recipe content
   - **Create meals table** for tracking meal instances:
     - Link to original recipe
     - Store actual instructions used (with modifications)
     - Capture user modifications/substitutions
     - Rating (1-5 stars)
     - Comments/notes from the meal
     - Date prepared
     - Thread context

4. **Complete Cooking Schema**

   ```sql
   -- Core recipe tables
   CREATE TABLE recipes (
       recipe_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       name TEXT NOT NULL,
       description TEXT,
       instructions JSONB NOT NULL CHECK (jsonb_typeof(instructions) = 'array'),
       prep_time INTEGER, -- minutes
       cook_time INTEGER, -- minutes
       servings INTEGER NOT NULL,
       generated_by_model TEXT -- NULL for human recipes, 'gpt-4', 'claude-3', etc for AI
   );

   CREATE TABLE ingredients (
       ingredient_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       name TEXT NOT NULL UNIQUE,
       category TEXT,
       default_unit_id UUID
   );

   CREATE TABLE units (
       unit_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       name TEXT NOT NULL UNIQUE,
       type TEXT CHECK (type IN ('volume', 'weight', 'count'))
   );

   CREATE TABLE recipe_ingredients (
       recipe_ingredient_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       recipe_id UUID REFERENCES recipes(recipe_id),
       ingredient_id UUID REFERENCES ingredients(ingredient_id),
       quantity DECIMAL NOT NULL,
       unit_id UUID REFERENCES units(unit_id),
       display_order INTEGER,
       preparation TEXT CHECK (
           preparation IN (
               'diced', 'minced', 'chopped', 'sliced', 'julienned',
               'grated', 'zested', 'crushed', 'mashed', 'whole',
               'halved', 'quartered', NULL
           )
       ),
       temperature TEXT CHECK (
           temperature IN (
               'room_temp', 'chilled', 'frozen', 'melted', 'softened', NULL
           )
       ),
       is_optional BOOLEAN DEFAULT false,
       is_used_in_multiple_steps BOOLEAN DEFAULT false
   );

   -- Tags
   CREATE TABLE tags (
       tag_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       name TEXT NOT NULL UNIQUE,
       color TEXT
   );

   CREATE TABLE recipe_tags (
       recipe_id UUID REFERENCES recipes(recipe_id),
       tag_id UUID REFERENCES tags(tag_id),
       PRIMARY KEY (recipe_id, tag_id)
   );

   -- Locations
   CREATE TABLE locations (
       location_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       name TEXT NOT NULL,
       parent_id UUID REFERENCES locations(location_id),
       location_type TEXT CHECK (
           location_type IN (
               'fridge', 'freezer', 'pantry', 'counter',
               'cabinet', 'drawer', 'shelf', 'bin', 'door'
           )
       )
   );

   -- Inventory
   CREATE TABLE inventory (
       inventory_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       ingredient_id UUID REFERENCES ingredients(ingredient_id),
       quantity DECIMAL NOT NULL,
       unit_id UUID REFERENCES units(unit_id),
       expiration_date DATE,
       location_id UUID REFERENCES locations(location_id)
   );

   -- Meal Planning
   CREATE TABLE meal_plans (
       meal_plan_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       name TEXT NOT NULL,
       start_date DATE NOT NULL,
       end_date DATE NOT NULL,
       notes TEXT
   );

   CREATE TABLE meal_plan_entries (
       meal_plan_entry_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       meal_plan_id UUID REFERENCES meal_plans(meal_plan_id),
       recipe_id UUID REFERENCES recipes(recipe_id),
       date DATE NOT NULL,
       meal_type TEXT CHECK (
           meal_type IN ('breakfast', 'lunch', 'dinner', 'snack')
       ),
       servings_override INTEGER
   );

   -- Meal Tracking (when meals are actually made)
   CREATE TABLE meals (
       meal_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       recipe_id UUID REFERENCES recipes(recipe_id),
       thread_id UUID REFERENCES threads(id) NOT NULL,
       prepared_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
       servings_made INTEGER,
       rating INTEGER CHECK (rating >= 1 AND rating <= 5),
       comments TEXT,
       modified_instructions JSONB, -- Track any modifications to instructions
       created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
   );

   CREATE INDEX idx_meals_thread_id ON meals(thread_id);
   CREATE INDEX idx_meals_recipe_id ON meals(recipe_id);
   CREATE INDEX idx_meals_prepared_at ON meals(prepared_at);

   -- Meal ingredients (what was actually used, including substitutions)
   CREATE TABLE meal_ingredients (
       meal_ingredient_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       meal_id UUID REFERENCES meals(meal_id) ON DELETE CASCADE,
       ingredient_id UUID REFERENCES ingredients(ingredient_id),
       quantity DECIMAL NOT NULL,
       unit_id UUID REFERENCES units(unit_id),
       original_ingredient_id UUID REFERENCES ingredients(ingredient_id), -- if substituted
       notes TEXT, -- e.g., "substituted with Greek yogurt"
       display_order INTEGER
   );

   CREATE INDEX idx_meal_ingredients_meal ON meal_ingredients(meal_id);
   CREATE INDEX idx_meal_ingredients_ingredient ON meal_ingredients(ingredient_id);
   ```

5. **Cooking Mode Tool Implementations**

   ```rust
   // Ingredient representation matching the schema
   pub struct IngredientInput {
       pub name: String,
       pub quantity: Decimal,
       pub unit_name: String,  // Will be resolved to unit_id
       pub preparation: Option<String>, // 'diced', 'minced', etc.
       pub temperature: Option<String>, // 'room_temp', 'chilled', etc.
       pub is_optional: bool,
   }
   
   // SaveRecipe: Store recipe with ingredients and instructions
   pub struct SaveRecipe {
       pub name: String,
       pub description: Option<String>,
       pub instructions: Vec<String>, // Stored as JSONB array
       pub ingredients: Vec<IngredientInput>,
       pub prep_time: Option<i32>,
       pub cook_time: Option<i32>,
       pub servings: i32,
       pub tags: Vec<String>,
       pub generated_by_model: Option<String>, // 'claude-3', 'gpt-4', etc.
   }
   
   // RecordMeal: Creates a new meal entry when user prepares a recipe
   pub struct RecordMeal {
       pub recipe_id: UUID,
       pub servings_made: Option<i32>,
       pub modified_instructions: Option<Vec<String>>, // If instructions were modified
       pub ingredients_used: Vec<MealIngredient>, // Actual ingredients including substitutions
   }
   
   pub struct MealIngredient {
       pub ingredient_name: String,
       pub quantity: Decimal,
       pub unit_name: String,
       pub original_ingredient_name: Option<String>, // If this was a substitution
       pub notes: Option<String>,
   }
   
   // RateMeal: Updates a meal with rating and comments
   pub struct RateMeal {
       pub meal_id: UUID,
       pub rating: i32,  // 1-5 stars
       pub comments: String,
   }
   
   // Inventory Management Tools
   pub struct UpdateInventory {
       pub ingredient_name: String,
       pub quantity: Decimal,
       pub unit_name: String,
       pub location: String, // 'fridge', 'pantry', etc.
       pub expiration_date: Option<Date>,
   }
   
   pub struct CheckInventory {
       pub recipe_id: Option<UUID>, // Check what's needed for a recipe
       pub location: Option<String>, // Filter by location
   }
   
   // Meal Planning Tools
   pub struct CreateMealPlan {
       pub name: String,
       pub start_date: Date,
       pub end_date: Date,
       pub notes: Option<String>,
   }
   
   pub struct AddToMealPlan {
       pub meal_plan_id: UUID,
       pub recipe_id: UUID,
       pub date: Date,
       pub meal_type: String, // 'breakfast', 'lunch', 'dinner', 'snack'
       pub servings_override: Option<i32>,
   }
   
   // Search and History Tools
   pub struct SearchRecipes {
       pub query: Option<String>,
       pub tags: Vec<String>,
       pub max_prep_time: Option<i32>,
       pub max_cook_time: Option<i32>,
       pub ingredients_include: Vec<String>,
       pub ingredients_exclude: Vec<String>,
   }
   
   pub struct GetMealHistory {
       pub thread_id: Option<UUID>,
       pub recipe_id: Option<UUID>,
       pub ingredient_id: Option<UUID>,
       pub min_rating: Option<i32>,
       pub date_range: Option<DateRange>,
   }
   
   // Tag Management Tools
   pub struct CreateTag {
       pub name: String,
       pub color: Option<String>,
   }
   
   pub struct AddTagToRecipe {
       pub recipe_id: UUID,
       pub tag_names: Vec<String>, // Can reference existing or create new tags
   }
   
   pub struct ListTags {
       pub filter: Option<String>, // Filter by name pattern
   }
   
   // Location Management Tools  
   pub struct CreateLocation {
       pub name: String,
       pub location_type: String, // 'fridge', 'freezer', 'pantry', etc.
       pub parent_name: Option<String>, // Parent location for hierarchy
   }
   
   pub struct ListLocations {
       pub location_type: Option<String>, // Filter by type
       pub parent_id: Option<UUID>, // List children of specific location
   }
   
   pub struct MoveInventory {
       pub inventory_id: UUID,
       pub new_location_name: String,
   }
   
   // Unit Management Tools
   pub struct CreateUnit {
       pub name: String,
       pub unit_type: String, // 'volume', 'weight', 'count'
   }
   
   pub struct ListUnits {
       pub unit_type: Option<String>, // Filter by type
   }
   
   // Meal Plan Query Tools
   pub struct GetMealPlan {
       pub meal_plan_id: Option<UUID>,
       pub date_range: Option<DateRange>, // Get plans for date range
       pub include_current: bool, // Include currently active plan
   }
   
   pub struct GetMealPlanShopping {
       pub meal_plan_id: UUID,
       pub check_inventory: bool, // Subtract what's already in inventory
   }
   
   pub struct GetUpcomingMeals {
       pub days_ahead: i32, // How many days to look ahead
       pub meal_type: Option<String>, // Filter by meal type
   }
   
   // Inventory Query Tools
   pub struct GetExpiringItems {
       pub days_ahead: i32, // Items expiring within X days
       pub location: Option<String>, // Filter by location
   }
   
   pub struct SearchInventory {
       pub ingredient_name: Option<String>,
       pub location: Option<String>,
       pub include_expired: bool,
   }
   
   // Recipe Analysis Tools
   pub struct AnalyzeRecipeNutrition {
       pub recipe_id: UUID,
       pub servings: Option<i32>, // Override default servings
   }
   
   pub struct SuggestRecipesByInventory {
       pub max_missing_ingredients: i32, // Recipes with at most X missing items
       pub tags: Vec<String>, // Filter by tags
   }
   
   pub struct GetRecipeStats {
       pub recipe_id: UUID, // Times made, average rating, etc.
   }
   ```
   
6. **Ingredient Parsing & Normalization**

   ```rust
   // Helper functions for ingredient management
   impl IngredientParser {
       // Parse raw ingredient text into structured data
       // "2 cups diced onions" -> { quantity: 2.0, unit: "cups", name: "onion", notes: "diced" }
       pub fn parse_ingredient(text: &str) -> Result<IngredientInput>;
       
       // Normalize ingredient names (plural to singular, common variations)
       // "onions" -> "onion", "bell peppers" -> "bell pepper"
       pub fn normalize_ingredient_name(name: &str) -> String;
       
       // Find or create ingredient in database
       pub async fn resolve_ingredient(name: &str) -> UUID;
   }
   ```

7. **Discord Channel Setup**
   - Create cooking channel
   - Map channel to cooking mode
   - Test thread creation with cooking mode
   - Test meal tracking workflow

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
- [ ] Complete cooking database schema migration
  - [ ] Recipes table with JSONB instructions
  - [ ] Ingredients master table
  - [ ] Units table with type constraints
  - [ ] Recipe ingredients with preparation/temperature
  - [ ] Tags and recipe_tags tables
  - [ ] Locations hierarchy table
  - [ ] Inventory tracking table
  - [ ] Meal plans and entries tables
  - [ ] Meals tracking table
  - [ ] Meal ingredients with substitution tracking
- [ ] Recipe Management Tools
  - [ ] SaveRecipe with full ingredient details
  - [ ] SearchRecipes with multiple filters
  - [ ] GetRecipeHistory
  - [ ] GetRecipeStats (times made, average rating)
  - [ ] AnalyzeRecipeNutrition
- [ ] Meal Tracking Tools
  - [ ] RecordMeal with ingredient substitutions
  - [ ] RateMeal with ratings and comments
  - [ ] GetMealHistory with filtering
- [ ] Tag Management Tools
  - [ ] CreateTag with color support
  - [ ] AddTagToRecipe
  - [ ] ListTags with filtering
- [ ] Location Management Tools
  - [ ] CreateLocation with hierarchy
  - [ ] ListLocations by type/parent
  - [ ] MoveInventory between locations
- [ ] Unit Management Tools
  - [ ] CreateUnit with type classification
  - [ ] ListUnits by type
- [ ] Inventory Management Tools
  - [ ] UpdateInventory with locations
  - [ ] CheckInventory for recipe requirements
  - [ ] SearchInventory with filters
  - [ ] GetExpiringItems alert system
- [ ] Meal Planning Tools
  - [ ] CreateMealPlan with date ranges
  - [ ] AddToMealPlan with meal types
  - [ ] GetMealPlan queries
  - [ ] GetUpcomingMeals forecast
  - [ ] GetMealPlanShopping list generation
- [ ] Smart Features
  - [ ] SuggestRecipesByInventory
- [ ] Ingredient parsing and normalization logic
- [ ] Unit resolution and conversion helpers
- [ ] Project Manager mode system prompt
- [ ] Basic project management tools
- [ ] Task/milestone storage schema
- [ ] Discord commands to set channel mode
- [ ] Mode indicator in thread viewer
- [ ] Tests for mode selection
- [ ] Tests for cooking mode
- [ ] Tests for project manager mode
- [ ] Documentation updates
