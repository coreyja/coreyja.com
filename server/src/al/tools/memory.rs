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
    /// The Discord user ID for the user (e.g., "1234567890")
    pub user_identifier: String,
    /// The memory content to save about this user. Should ideally include the user's preferred name or username.
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
    const DESCRIPTION: &'static str = "Save or update a memory about a specific user using their Discord user ID. \
        This allows you to remember important information about users across conversations. \
        Memories are stored with a Discord user ID and can be retrieved later. \
        The memory content should ideally include the user's preferred name or username for easy reference. \
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
    /// The Discord user ID for the user whose memory you want to read (e.g., "1234567890")
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
    const DESCRIPTION: &'static str =
        "Read a memory about a specific user using their Discord user ID. \
        This allows you to retrieve previously saved information about users. \
        Provide the Discord user ID to look up their memory.";

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

#[derive(Clone, Debug)]
pub struct AppendUserMemory;

impl AppendUserMemory {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AppendUserMemoryInput {
    /// The Discord user ID for the user (e.g., "1234567890")
    pub user_identifier: String,
    /// The new content to append to this user's memory. Should ideally include the user's preferred name or username if not already present.
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendUserMemoryOutput {
    pub success: bool,
    pub message: String,
}

#[async_trait::async_trait]
impl Tool for AppendUserMemory {
    const NAME: &'static str = "append_user_memory";
    const DESCRIPTION: &'static str = "Append new information to a user's existing memory using their Discord user ID. \
        This safely adds content to what you already know about a user without overwriting existing information. \
        If no memory exists for this user, a new memory will be created. \
        The memory content should ideally include the user's preferred name or username for easy reference. \
        Use this tool when you want to add new facts about a user while preserving what you already know.";

    type ToolInput = AppendUserMemoryInput;
    type ToolOutput = AppendUserMemoryOutput;

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
            // Append to existing memory with a newline separator
            let updated_content = format!("{}\n{}", memory.content, input.content);
            MemoryBlock::update_content(pool, memory.memory_block_id, updated_content).await?;
            Ok(AppendUserMemoryOutput {
                success: true,
                message: format!(
                    "Successfully appended to memory for user '{}'",
                    input.user_identifier
                ),
            })
        } else {
            // Create new memory if none exists
            MemoryBlock::create(
                pool,
                "person".to_string(),
                input.user_identifier.clone(),
                input.content,
            )
            .await?;
            Ok(AppendUserMemoryOutput {
                success: true,
                message: format!(
                    "Successfully created new memory for user '{}'",
                    input.user_identifier
                ),
            })
        }
    }
}
