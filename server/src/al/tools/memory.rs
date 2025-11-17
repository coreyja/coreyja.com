use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::al::tools::{ThreadContext, Tool};
use crate::memory::blocks::MemoryBlock;
use crate::AppState;

#[derive(Clone, Debug)]
pub struct SaveUserMemory;

impl SaveUserMemory {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SaveUserMemoryInput {
    /// The identifier for the user (e.g., Discord username#discriminator, or any unique user identifier)
    pub user_identifier: String,
    /// The memory content to save about this user
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveUserMemoryOutput {
    pub success: bool,
    pub message: String,
}

#[async_trait::async_trait]
impl Tool for SaveUserMemory {
    const NAME: &'static str = "save_user_memory";
    const DESCRIPTION: &'static str = "Save or update a memory about a specific user. \
        This allows you to remember important information about users across conversations. \
        Memories are stored with a user identifier and can be retrieved later. \
        If a memory already exists for this user, it will be updated with the new content.";

    type ToolInput = SaveUserMemoryInput;
    type ToolOutput = SaveUserMemoryOutput;

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        _context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        let pool = &app_state.db;

        // Check if a memory already exists for this user
        let existing_memory = MemoryBlock::find_by_type_and_identifier(
            pool,
            "person".to_string(),
            input.user_identifier.clone(),
        )
        .await?;

        if let Some(memory) = existing_memory {
            // Update existing memory
            MemoryBlock::update_content(pool, memory.memory_block_id, input.content).await?;
            Ok(SaveUserMemoryOutput {
                success: true,
                message: format!(
                    "Successfully updated memory for user '{}'",
                    input.user_identifier
                ),
            })
        } else {
            // Create new memory
            MemoryBlock::create(
                pool,
                "person".to_string(),
                input.user_identifier.clone(),
                input.content,
            )
            .await?;
            Ok(SaveUserMemoryOutput {
                success: true,
                message: format!(
                    "Successfully created memory for user '{}'",
                    input.user_identifier
                ),
            })
        }
    }
}

#[derive(Clone, Debug)]
pub struct ReadUserMemory;

impl ReadUserMemory {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReadUserMemoryInput {
    /// The identifier for the user whose memory you want to read (e.g., Discord username#discriminator)
    pub user_identifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadUserMemoryOutput {
    pub found: bool,
    pub content: Option<String>,
    pub message: String,
}

#[async_trait::async_trait]
impl Tool for ReadUserMemory {
    const NAME: &'static str = "read_user_memory";
    const DESCRIPTION: &'static str = "Read a memory about a specific user. \
        This allows you to retrieve previously saved information about users. \
        Provide the user identifier to look up their memory.";

    type ToolInput = ReadUserMemoryInput;
    type ToolOutput = ReadUserMemoryOutput;

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        _context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        let pool = &app_state.db;

        let memory = MemoryBlock::find_by_type_and_identifier(
            pool,
            "person".to_string(),
            input.user_identifier.clone(),
        )
        .await?;

        match memory {
            Some(memory_block) => Ok(ReadUserMemoryOutput {
                found: true,
                content: Some(memory_block.content),
                message: format!(
                    "Successfully retrieved memory for user '{}'",
                    input.user_identifier
                ),
            }),
            None => Ok(ReadUserMemoryOutput {
                found: false,
                content: None,
                message: format!("No memory found for user '{}'", input.user_identifier),
            }),
        }
    }
}
