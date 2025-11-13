use color_eyre::eyre::{eyre, Result};
use db::agentic_threads::{Stitch, Thread, ThreadType};
use db::discord_threads::DiscordThreadMetadata;
use db::linear_threads::LinearThreadMetadata;
use sqlx::PgPool;
use uuid::Uuid;

use crate::memory::MemoryManager;

pub struct DiscordMetadata {
    pub discord_thread_id: String,
    pub channel_id: String,
    pub guild_id: String,
    pub created_by: String,
    pub thread_name: String,
}

pub struct LinearMetadata {
    pub session_id: String,
    pub workspace_id: String,
    pub issue_id: Option<String>,
    pub issue_title: Option<String>,
    pub project_id: Option<String>,
    pub team_id: Option<String>,
    pub created_by_user_id: String,
}

pub struct ThreadBuilder {
    pool: PgPool,
    goal: String,
    thread_type: ThreadType,
    branching_stitch_id: Option<Uuid>,
    discord_metadata: Option<DiscordMetadata>,
    linear_metadata: Option<LinearMetadata>,
    agent_id: crate::agent_config::AgentId,
    persona: Option<String>,
}

impl ThreadBuilder {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            goal: String::new(),
            thread_type: ThreadType::Autonomous,
            branching_stitch_id: None,
            discord_metadata: None,
            linear_metadata: None,
            agent_id: crate::agent_config::DEFAULT_AGENT_ID,
            persona: None,
        }
    }

    pub fn with_goal(mut self, goal: impl Into<String>) -> Self {
        self.goal = goal.into();
        self
    }

    pub fn autonomous(mut self) -> Self {
        self.thread_type = ThreadType::Autonomous;
        self
    }

    pub fn interactive_discord(mut self, metadata: DiscordMetadata) -> Self {
        self.thread_type = ThreadType::Interactive;
        self.discord_metadata = Some(metadata);
        self
    }

    pub fn interactive_linear(mut self, metadata: LinearMetadata) -> Self {
        self.thread_type = ThreadType::Interactive;
        self.linear_metadata = Some(metadata);
        self
    }

    pub fn child_of(mut self, parent_stitch_id: Uuid) -> Self {
        self.branching_stitch_id = Some(parent_stitch_id);
        self
    }

    pub fn with_agent(mut self, agent_id: crate::agent_config::AgentId) -> Self {
        self.agent_id = agent_id;
        self
    }

    pub fn with_persona(mut self, persona: impl Into<String>) -> Self {
        self.persona = Some(persona.into());
        self
    }

    pub async fn build(self) -> Result<Thread> {
        // Validate
        if self.goal.is_empty() {
            return Err(eyre!("Thread goal cannot be empty"));
        }

        // Validate metadata if thread type is Interactive
        if self.thread_type == ThreadType::Interactive
            && self.discord_metadata.is_none()
            && self.linear_metadata.is_none()
        {
            return Err(eyre!(
                "Interactive threads must have either Discord or Linear metadata"
            ));
        }

        // Create the thread using the unified method
        let thread = Thread::create(
            &self.pool,
            self.goal,
            self.branching_stitch_id,
            Some(self.thread_type),
            self.agent_id.to_string(),
        )
        .await?;

        // Extract person identifier from Discord metadata if present
        let person_identifier = self
            .discord_metadata
            .as_ref()
            .map(|meta| meta.created_by.clone());

        // Create Discord metadata if this is a Discord interactive thread
        if let Some(discord_meta) = self.discord_metadata {
            DiscordThreadMetadata::create(
                &self.pool,
                thread.thread_id,
                discord_meta.discord_thread_id,
                discord_meta.channel_id,
                discord_meta.guild_id,
                discord_meta.created_by,
                discord_meta.thread_name,
            )
            .await?;
        }

        // Create Linear metadata if this is a Linear interactive thread
        if let Some(linear_meta) = self.linear_metadata {
            LinearThreadMetadata::create(
                &self.pool,
                thread.thread_id,
                linear_meta.session_id,
                linear_meta.workspace_id,
                linear_meta.issue_id,
                linear_meta.issue_title,
                linear_meta.project_id,
                linear_meta.team_id,
                linear_meta.created_by_user_id,
            )
            .await?;
        }

        let memory_manager = MemoryManager::new(self.pool.clone());
        let system_prompt = memory_manager
            .generate_system_prompt(&thread, person_identifier, self.persona)
            .await?;

        // Create system prompt stitch
        Stitch::create_system_prompt(&self.pool, thread.thread_id, system_prompt).await?;

        Ok(thread)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::blocks::MemoryBlock;

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_thread_builder_extracts_discord_author_for_person_memory(pool: PgPool) {
        // Create person memory for a Discord user
        let person_content = "Test user who loves Rust programming.";
        MemoryBlock::create(
            &pool,
            "person".to_string(),
            "testuser#1234".to_string(),
            person_content.to_string(),
        )
        .await
        .unwrap();

        // Create Discord metadata with the same user (use valid Discord snowflake IDs)
        let discord_meta = DiscordMetadata {
            discord_thread_id: "123456789012345678".to_string(),
            channel_id: "234567890123456789".to_string(),
            guild_id: "345678901234567890".to_string(),
            created_by: "testuser#1234".to_string(),
            thread_name: "Test Thread".to_string(),
        };

        // Build a Discord thread
        let thread = ThreadBuilder::new(pool.clone())
            .with_goal("Help with Rust code")
            .interactive_discord(discord_meta)
            .build()
            .await
            .unwrap();

        // Fetch the system prompt stitch
        let stitches = db::agentic_threads::Stitch::get_by_thread_ordered(&pool, thread.thread_id)
            .await
            .unwrap();

        // Find system prompt stitch
        let system_stitch = stitches
            .iter()
            .find(|s| s.stitch_type == db::agentic_threads::StitchType::SystemPrompt)
            .expect("Should have system prompt stitch");

        // Extract content from llm_request
        let content = system_stitch
            .llm_request
            .as_ref()
            .and_then(|r| r.get("text"))
            .and_then(|t| t.as_str())
            .expect("Should have text in llm_request");

        // Verify person memory was injected
        assert!(content.contains("--- PERSON MEMORY BLOCK ---"));
        assert!(content.contains(person_content));
        assert!(content.contains("Test user who loves Rust programming"));
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_thread_builder_handles_missing_person_memory_gracefully(pool: PgPool) {
        // Create Discord metadata for user without person memory (use valid Discord snowflake IDs)
        let discord_meta = DiscordMetadata {
            discord_thread_id: "999888777666555444".to_string(),
            channel_id: "888777666555444333".to_string(),
            guild_id: "777666555444333222".to_string(),
            created_by: "unknownuser#9999".to_string(),
            thread_name: "Test Thread".to_string(),
        };

        // Build thread - should not fail
        let thread = ThreadBuilder::new(pool.clone())
            .with_goal("Help with code")
            .interactive_discord(discord_meta)
            .build()
            .await
            .unwrap();

        // Fetch the system prompt stitch
        let stitches = db::agentic_threads::Stitch::get_by_thread_ordered(&pool, thread.thread_id)
            .await
            .unwrap();

        let system_stitch = stitches
            .iter()
            .find(|s| s.stitch_type == db::agentic_threads::StitchType::SystemPrompt)
            .expect("Should have system prompt stitch");

        // Extract content from llm_request
        let content = system_stitch
            .llm_request
            .as_ref()
            .and_then(|r| r.get("text"))
            .and_then(|t| t.as_str())
            .expect("Should have text in llm_request");

        // Should NOT contain person memory block
        assert!(!content.contains("--- PERSON MEMORY BLOCK ---"));

        // Should still have base instructions
        assert!(content.contains("AI assistant"));
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_thread_builder_autonomous_thread_no_person_memory(pool: PgPool) {
        // Build autonomous thread (no Discord metadata)
        let thread = ThreadBuilder::new(pool.clone())
            .with_goal("Autonomous task")
            .autonomous()
            .build()
            .await
            .unwrap();

        // Fetch the system prompt stitch
        let stitches = db::agentic_threads::Stitch::get_by_thread_ordered(&pool, thread.thread_id)
            .await
            .unwrap();

        let system_stitch = stitches
            .iter()
            .find(|s| s.stitch_type == db::agentic_threads::StitchType::SystemPrompt)
            .expect("Should have system prompt stitch");

        // Extract content from llm_request
        let content = system_stitch
            .llm_request
            .as_ref()
            .and_then(|r| r.get("text"))
            .and_then(|t| t.as_str())
            .expect("Should have text in llm_request");

        // Should NOT contain person memory block (no Discord metadata)
        assert!(!content.contains("--- PERSON MEMORY BLOCK ---"));

        // Should have base instructions
        assert!(content.contains("AI assistant"));
    }
}
