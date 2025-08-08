# Phase 1: Core Mode Infrastructure Implementation

## Overview
Implement the foundational mode system that allows AI agent threads to operate in different specialized modes (General, Cooking, ProjectManager) based on Discord channel configuration.

## Database Migrations

### 1. Create migration for threads table

Create file: `db/migrations/[timestamp]_add_mode_to_threads.up.sql`

```sql
-- Add mode column to threads table
ALTER TABLE threads ADD COLUMN mode text;

-- Add check constraint for valid modes
ALTER TABLE threads ADD CONSTRAINT threads_mode_check 
    CHECK (mode IN ('general', 'cooking', 'project_manager'));

-- Update existing threads to have 'general' mode
UPDATE threads SET mode = 'general' WHERE mode IS NULL;

-- Make mode NOT NULL with default
ALTER TABLE threads ALTER COLUMN mode SET NOT NULL;
ALTER TABLE threads ALTER COLUMN mode SET DEFAULT 'general';
```

Create file: `db/migrations/[timestamp]_add_mode_to_threads.down.sql`

```sql
-- Remove mode column and constraint from threads table
ALTER TABLE threads DROP CONSTRAINT IF EXISTS threads_mode_check;
ALTER TABLE threads DROP COLUMN IF EXISTS mode;
```

### 2. Create migration for DiscordChannels table

Create file: `db/migrations/[timestamp]_add_mode_to_discord_channels.up.sql`

```sql
-- Add mode column to DiscordChannels table
ALTER TABLE DiscordChannels ADD COLUMN mode text;

-- Add check constraint for valid modes
ALTER TABLE DiscordChannels ADD CONSTRAINT discord_channels_mode_check 
    CHECK (mode IN ('general', 'cooking', 'project_manager'));

-- Update existing channels to have 'general' mode
UPDATE DiscordChannels SET mode = 'general' WHERE mode IS NULL;

-- Make mode NOT NULL with default
ALTER TABLE DiscordChannels ALTER COLUMN mode SET NOT NULL;
ALTER TABLE DiscordChannels ALTER COLUMN mode SET DEFAULT 'general';
```

Create file: `db/migrations/[timestamp]_add_mode_to_discord_channels.down.sql`

```sql
-- Remove mode column and constraint from DiscordChannels table
ALTER TABLE DiscordChannels DROP CONSTRAINT IF EXISTS discord_channels_mode_check;
ALTER TABLE DiscordChannels DROP COLUMN IF EXISTS mode;
```

## Rust Code Implementation

### 1. Add ThreadMode enum to `db/src/agentic_threads/mod.rs`

Add after the existing enums (StitchType, ThreadType, ThreadStatus):

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ThreadMode {
    General,
    Cooking,
    ProjectManager,
}

impl fmt::Display for ThreadMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThreadMode::General => write!(f, "general"),
            ThreadMode::Cooking => write!(f, "cooking"),
            ThreadMode::ProjectManager => write!(f, "project_manager"),
        }
    }
}

impl std::str::FromStr for ThreadMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "general" => Ok(ThreadMode::General),
            "cooking" => Ok(ThreadMode::Cooking),
            "project_manager" => Ok(ThreadMode::ProjectManager),
            _ => Err(format!("Unknown thread mode: {s}")),
        }
    }
}

impl From<String> for ThreadMode {
    fn from(s: String) -> Self {
        s.parse().expect("Invalid thread mode")
    }
}

impl Default for ThreadMode {
    fn default() -> Self {
        ThreadMode::General
    }
}
```

### 2. Update Thread struct in `db/src/agentic_threads/mod.rs`

Find the Thread struct definition and add the mode field:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Thread {
    pub thread_id: Uuid,
    pub goal: String,
    pub status: ThreadStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub thread_type: ThreadType,
    pub metadata: JsonValue,
    pub mode: ThreadMode,  // Add this field
}
```

### 3. Update Thread::create method in `db/src/agentic_threads/mod.rs`

Update the create method to accept and store mode:

```rust
impl Thread {
    pub async fn create(
        pool: &PgPool,
        goal: String,
        branching_stitch_id: Option<Uuid>,
        thread_type: Option<ThreadType>,
        mode: Option<ThreadMode>,  // Add this parameter
    ) -> Result<Self> {
        let thread_type = thread_type.unwrap_or(ThreadType::Autonomous);
        let mode = mode.unwrap_or_default();  // Use default (General) if not specified
        
        let thread = sqlx::query_as!(
            Thread,
            r#"
            INSERT INTO threads (goal, status, thread_type, mode)
            VALUES ($1, $2, $3, $4)
            RETURNING 
                thread_id,
                goal,
                status as "status: ThreadStatus",
                created_at,
                updated_at,
                thread_type as "thread_type: ThreadType",
                metadata,
                mode as "mode: ThreadMode"
            "#,
            goal,
            ThreadStatus::Pending as ThreadStatus,
            thread_type as ThreadType,
            mode as ThreadMode,
        )
        .fetch_one(pool)
        .await?;

        // ... rest of the method remains the same
    }
}
```

### 4. Update DiscordChannel struct in `db/src/lib.rs`

Add the mode field to the DiscordChannel struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DiscordChannel {
    pub discord_channel_id: Uuid,
    pub channel_name: Option<String>,
    pub channel_topic: Option<String>,
    pub channel_id: String,
    pub purpose: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub mode: ThreadMode,  // Add this field
}
```

### 5. Update ThreadBuilder in `server/src/agentic_threads/builder.rs`

Add mode support to the builder:

```rust
use db::agentic_threads::{Stitch, Thread, ThreadType, ThreadMode};

pub struct ThreadBuilder {
    pool: PgPool,
    goal: String,
    thread_type: ThreadType,
    branching_stitch_id: Option<Uuid>,
    discord_metadata: Option<DiscordMetadata>,
    mode: ThreadMode,  // Add this field
}

impl ThreadBuilder {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            goal: String::new(),
            thread_type: ThreadType::Autonomous,
            branching_stitch_id: None,
            discord_metadata: None,
            mode: ThreadMode::default(),  // Default to General
        }
    }

    // Add method to set mode
    pub fn with_mode(mut self, mode: ThreadMode) -> Self {
        self.mode = mode;
        self
    }

    pub async fn build(self) -> Result<Thread> {
        // ... existing validation ...

        // Create the thread with mode
        let thread = Thread::create(
            &self.pool,
            self.goal,
            self.branching_stitch_id,
            Some(self.thread_type),
            Some(self.mode),  // Pass the mode
        )
        .await?;

        // ... rest of the method remains the same
    }
}
```

### 6. Update Discord thread creation in `server/src/jobs/discord_message_processor.rs`

When creating a thread from Discord, look up the channel's mode:

```rust
// In the process method where thread is created
// First, get the channel from the database
let channel = sqlx::query_as!(
    DiscordChannel,
    r#"
    SELECT 
        discord_channel_id,
        channel_name,
        channel_topic,
        channel_id,
        purpose,
        created_at,
        updated_at,
        mode as "mode: ThreadMode"
    FROM DiscordChannels 
    WHERE channel_id = $1
    "#,
    channel_id
)
.fetch_optional(&state.pool)
.await?;

// Use the channel's mode or default to General
let mode = channel.map(|c| c.mode).unwrap_or_default();

// Create the thread with the mode
let thread = ThreadBuilder::new(state.pool.clone())
    .with_goal(goal)
    .with_mode(mode)  // Set the mode from channel
    .interactive_discord(discord_metadata)
    .build()
    .await?;
```

### 7. Update system prompt generation in `server/src/memory/prompts.rs`

Add mode-specific instructions to the prompt generator:

```rust
use db::agentic_threads::ThreadMode;

impl PromptGenerator {
    pub async fn generate_system_prompt(pool: &PgPool, thread: &Thread) -> Result<String> {
        // Base instructions (always included)
        let mut system_content = Self::base_instructions().to_string();

        // Add thread goal
        write!(system_content, "\nCurrent goal: {}\n", thread.goal)?;

        // Add mode-specific instructions
        let mode_instructions = Self::generate_mode_instructions(&thread.mode);
        if !mode_instructions.is_empty() {
            system_content.push_str("\n--- MODE-SPECIFIC INSTRUCTIONS ---\n");
            system_content.push_str(mode_instructions);
            system_content.push_str("\n--- END MODE-SPECIFIC INSTRUCTIONS ---\n");
        }

        // Add persona if available
        let persona = MemoryBlock::get_persona(pool).await?;
        if let Some(persona_block) = persona {
            system_content.push_str("\n--- PERSONA MEMORY BLOCK ---\n");
            system_content.push_str(&persona_block.content);
            system_content.push_str("\n--- END PERSONA MEMORY BLOCK ---\n");
        }

        // Add context-specific instructions for Discord
        if thread.thread_type == ThreadType::Interactive {
            system_content.push_str(Self::discord_instructions());
        }

        Ok(system_content)
    }

    pub fn generate_mode_instructions(mode: &ThreadMode) -> &'static str {
        match mode {
            ThreadMode::Cooking => {
                "You are a culinary assistant specializing in:\n\
                - Recipe suggestions and modifications\n\
                - Meal planning and preparation\n\
                - Ingredient substitutions\n\
                - Cooking techniques and tips\n\
                - Dietary accommodations\n\
                Track recipes discussed and modifications made."
            },
            ThreadMode::ProjectManager => {
                "You are a project management assistant specializing in:\n\
                - Task breakdown and prioritization\n\
                - Timeline and milestone planning\n\
                - Resource allocation and dependency tracking\n\
                - Risk assessment and mitigation strategies\n\
                - Progress monitoring and status reporting\n\
                Help organize work into actionable tasks with clear deliverables."
            },
            ThreadMode::General => {
                "" // General mode uses standard base instructions only
            }
        }
    }
}
```

### 8. Update thread API response in `server/src/http_server/api/threads.rs`

Include mode in the API response:

```rust
#[derive(Debug, Serialize)]
pub struct ThreadResponse {
    pub thread_id: Uuid,
    pub goal: String,
    pub status: ThreadStatus,
    pub thread_type: ThreadType,
    pub mode: ThreadMode,  // Add this field
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub stitches: Vec<StitchResponse>,
}

impl From<Thread> for ThreadResponse {
    fn from(thread: Thread) -> Self {
        Self {
            thread_id: thread.thread_id,
            goal: thread.goal,
            status: thread.status,
            thread_type: thread.thread_type,
            mode: thread.mode,  // Include mode
            created_at: thread.created_at,
            updated_at: thread.updated_at,
            stitches: vec![],
        }
    }
}
```

## Testing

### 1. Run database migrations
```bash
cd db
cargo sqlx migrate run
```

### 2. Regenerate SQLx query metadata
```bash
./scripts/auto-fix-all.sh
```

### 3. Build and test
```bash
./scripts/dev-build.sh
cargo test --workspace
```

### 4. Manual testing checklist
- [ ] Create a new thread via API - verify it gets 'general' mode
- [ ] Create a Discord thread - verify it inherits channel's mode
- [ ] Check existing threads have been migrated to 'general' mode
- [ ] Verify mode appears in thread viewer API responses
- [ ] Test system prompt includes mode-specific instructions

## Discord Commands (Future)

For managing channel modes, you'll need Discord slash commands:
- `/set-channel-mode [mode]` - Set the mode for current channel
- `/get-channel-mode` - Display current channel's mode

These will be implemented in a later phase.

## Verification Steps

1. **Database verification:**
   ```sql
   -- Check threads have mode column
   SELECT thread_id, goal, mode FROM threads LIMIT 5;
   
   -- Check DiscordChannels have mode column
   SELECT channel_id, channel_name, mode FROM DiscordChannels LIMIT 5;
   ```

2. **API verification:**
   - GET `/api/threads/{id}` should include mode field
   - New threads should default to 'general' mode

3. **System prompt verification:**
   - Cooking mode threads should have culinary instructions
   - Project manager mode threads should have PM instructions
   - General mode should have no additional instructions

## Rollback Plan

If issues arise, rollback using the down migrations:

```bash
cd db
cargo sqlx migrate revert  # This will run the .down.sql files
```

Or manually run the down migrations:
- `db/migrations/[timestamp]_add_mode_to_threads.down.sql`
- `db/migrations/[timestamp]_add_mode_to_discord_channels.down.sql`

## Next Steps

After Phase 1 is complete and tested:
1. Phase 2: Implement Cooking mode with recipe tools
2. Phase 3: Implement Project Manager mode with task tools
3. Phase 4: Enhanced cooking features
4. Phase 5: Mode management UI

## Notes

- All existing functionality is preserved - 'general' mode behaves exactly like the current system
- Mode system is extensible - new modes can be added by updating the enum and constraints
- Channel-to-mode mapping enables automatic mode selection based on Discord context
- System prompts are dynamically generated based on mode for specialized behavior