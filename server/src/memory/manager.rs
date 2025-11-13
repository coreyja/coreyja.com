use color_eyre::Result;
use db::agentic_threads::Thread;
use sqlx::PgPool;

use super::blocks::MemoryBlock;
use super::prompts::PromptGenerator;

/// Unified interface for memory management
#[derive(Debug)]
pub struct MemoryManager {
    pool: PgPool,
}

impl MemoryManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Retrieve current persona configuration
    pub async fn get_persona(&self) -> Result<Option<String>> {
        let persona = MemoryBlock::get_persona(&self.pool).await?;
        Ok(persona.map(|p| p.content))
    }

    /// Update persona content
    pub async fn update_persona(&self, content: String) -> Result<MemoryBlock> {
        // First try to find existing persona (type="persona", identifier="default")
        let existing = MemoryBlock::get_persona(&self.pool).await?;

        if let Some(persona) = existing {
            // Update existing persona
            MemoryBlock::update_content(&self.pool, persona.memory_block_id, content)
                .await?
                .ok_or_else(|| color_eyre::eyre::eyre!("Failed to update persona"))
        } else {
            // Create new persona with type="persona", identifier="default"
            MemoryBlock::create(
                &self.pool,
                "persona".to_string(),
                "default".to_string(),
                content,
            )
            .await
        }
    }

    /// Build complete system prompt with persona and optional person memory
    pub async fn generate_system_prompt(
        &self,
        thread: &Thread,
        person_identifier: Option<String>,
        persona: Option<String>,
    ) -> Result<String> {
        PromptGenerator::generate_system_prompt(&self.pool, thread, person_identifier, persona)
            .await
    }

    /// Get reference to the database pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use db::agentic_threads::{Thread, ThreadType};

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_memory_manager_new_and_pool_access(pool: PgPool) {
        let manager = MemoryManager::new(pool.clone());

        // Verify we can access the pool
        let pool_ref = manager.pool();
        assert_eq!(pool_ref.size() as usize, pool.size() as usize);
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_memory_manager_get_persona(pool: PgPool) {
        let manager = MemoryManager::new(pool.clone());

        // Initially no persona
        let persona = manager.get_persona().await.unwrap();
        assert!(persona.is_none());

        // Create a persona
        let content = "I am a test persona";
        MemoryBlock::create(
            &pool,
            "persona".to_string(),
            "default".to_string(),
            content.to_string(),
        )
        .await
        .unwrap();

        // Now should have persona
        let persona = manager.get_persona().await.unwrap();
        assert_eq!(persona, Some(content.to_string()));
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_memory_manager_update_persona(pool: PgPool) {
        let manager = MemoryManager::new(pool.clone());

        // Create new persona
        let content = "New persona content";
        let created = manager.update_persona(content.to_string()).await.unwrap();
        assert_eq!(created.content, content);

        // Update existing persona
        let updated_content = "Updated persona content";
        let updated = manager
            .update_persona(updated_content.to_string())
            .await
            .unwrap();
        assert_eq!(updated.content, updated_content);
        assert_eq!(updated.memory_block_id, created.memory_block_id);
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_memory_manager_generate_system_prompt(pool: PgPool) {
        let manager = MemoryManager::new(pool.clone());

        // Create test threads
        let autonomous_thread = Thread::create(
            &pool,
            "Autonomous goal".to_string(),
            None,
            Some(ThreadType::Autonomous),
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();
        let interactive_thread = Thread::create(
            &pool,
            "Interactive goal".to_string(),
            None,
            Some(ThreadType::Interactive),
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();

        // Test without persona, non-Discord (pass None for person_identifier)
        let prompt = manager
            .generate_system_prompt(&autonomous_thread, None, None)
            .await
            .unwrap();
        assert!(prompt.contains("AI assistant"));
        assert!(prompt.contains("Current goal: Autonomous goal"));
        assert!(!prompt.contains("Discord"));

        // Test with Discord context (pass None for person_identifier)
        let discord_prompt = manager
            .generate_system_prompt(&interactive_thread, None, None)
            .await
            .unwrap();
        assert!(discord_prompt.contains("AI assistant"));
        assert!(discord_prompt.contains("Current goal: Interactive goal"));
        assert!(discord_prompt.contains("Discord"));

        // Add persona and test again (pass None for person_identifier)
        let persona_content = "I am a test persona for the manager";
        manager
            .update_persona(persona_content.to_string())
            .await
            .unwrap();

        let prompt_with_persona = manager
            .generate_system_prompt(&autonomous_thread, None, None)
            .await
            .unwrap();
        assert!(prompt_with_persona.contains(persona_content));
        assert!(prompt_with_persona.contains("--- PERSONA MEMORY BLOCK ---"));
    }

    // Task Group 5 Integration Test

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_end_to_end_discord_thread_with_person_and_persona_memory(pool: PgPool) {
        // Integration test: End-to-end flow with both persona and person memory
        let manager = MemoryManager::new(pool.clone());

        // 1. Setup: Create persona block
        let persona_content =
            "I am Al, a friendly AI assistant who loves helping people with their projects.";
        manager
            .update_persona(persona_content.to_string())
            .await
            .unwrap();

        // 2. Setup: Create person memory for a Discord user
        let discord_username = "alice#1234";
        let person_content = "Alice is a senior software engineer who specializes in Rust and distributed systems. She prefers detailed technical explanations.";
        MemoryBlock::create(
            &pool,
            "person".to_string(),
            discord_username.to_string(),
            person_content.to_string(),
        )
        .await
        .unwrap();

        // 3. Create an interactive thread (simulating Discord)
        let thread = Thread::create(
            &pool,
            "Help Alice debug her Rust async code".to_string(),
            None,
            Some(ThreadType::Interactive),
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();

        // 4. Generate system prompt with person identifier
        let system_prompt = manager
            .generate_system_prompt(&thread, Some(discord_username.to_string()), None)
            .await
            .unwrap();

        // 5. Verify the prompt contains all expected elements

        // Base instructions
        assert!(system_prompt.contains("AI assistant"));
        assert!(system_prompt.contains("helpful"));

        // Thread goal
        assert!(system_prompt.contains("Current goal: Help Alice debug her Rust async code"));

        // Persona block
        assert!(system_prompt.contains("--- PERSONA MEMORY BLOCK ---"));
        assert!(system_prompt.contains(persona_content));
        assert!(system_prompt.contains("--- END PERSONA MEMORY BLOCK ---"));

        // Person memory block
        assert!(system_prompt.contains("--- PERSON MEMORY BLOCK ---"));
        assert!(system_prompt.contains(person_content));
        assert!(system_prompt.contains("--- END PERSON MEMORY BLOCK ---"));

        // Discord instructions
        assert!(system_prompt.contains("Discord"));
        assert!(system_prompt.contains("2000 characters"));

        // 6. Verify order: persona comes before person memory
        let persona_pos = system_prompt.find("--- PERSONA MEMORY BLOCK ---").unwrap();
        let person_pos = system_prompt.find("--- PERSON MEMORY BLOCK ---").unwrap();
        assert!(
            persona_pos < person_pos,
            "Persona should come before person memory"
        );

        // 7. Test with different user (no person memory)
        let thread2 = Thread::create(
            &pool,
            "Help Bob with his code".to_string(),
            None,
            Some(ThreadType::Interactive),
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();

        let system_prompt2 = manager
            .generate_system_prompt(&thread2, Some("bob#9999".to_string()), None)
            .await
            .unwrap();

        // Should have persona but NOT person memory
        assert!(system_prompt2.contains("--- PERSONA MEMORY BLOCK ---"));
        assert!(!system_prompt2.contains("--- PERSON MEMORY BLOCK ---"));
        assert!(system_prompt2.contains(persona_content));
        assert!(!system_prompt2.contains(person_content));
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_memory_manager_with_multiple_person_memories(pool: PgPool) {
        // Integration test: Multiple person memories don't interfere with each other
        let manager = MemoryManager::new(pool.clone());

        // Create multiple person memories
        MemoryBlock::create(
            &pool,
            "person".to_string(),
            "user1#1111".to_string(),
            "User 1 prefers concise answers.".to_string(),
        )
        .await
        .unwrap();

        MemoryBlock::create(
            &pool,
            "person".to_string(),
            "user2#2222".to_string(),
            "User 2 prefers detailed explanations with examples.".to_string(),
        )
        .await
        .unwrap();

        MemoryBlock::create(
            &pool,
            "person".to_string(),
            "user3#3333".to_string(),
            "User 3 is learning Rust and needs beginner-friendly explanations.".to_string(),
        )
        .await
        .unwrap();

        // Create threads for each user
        let thread1 = Thread::create(
            &pool,
            "Goal 1".to_string(),
            None,
            Some(ThreadType::Interactive),
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();
        let thread2 = Thread::create(
            &pool,
            "Goal 2".to_string(),
            None,
            Some(ThreadType::Interactive),
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();
        let thread3 = Thread::create(
            &pool,
            "Goal 3".to_string(),
            None,
            Some(ThreadType::Interactive),
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();

        // Generate prompts for each user
        let prompt1 = manager
            .generate_system_prompt(&thread1, Some("user1#1111".to_string()), None)
            .await
            .unwrap();
        let prompt2 = manager
            .generate_system_prompt(&thread2, Some("user2#2222".to_string()), None)
            .await
            .unwrap();
        let prompt3 = manager
            .generate_system_prompt(&thread3, Some("user3#3333".to_string()), None)
            .await
            .unwrap();

        // Verify each prompt contains ONLY the correct person memory
        assert!(prompt1.contains("User 1 prefers concise answers"));
        assert!(!prompt1.contains("User 2 prefers detailed explanations"));
        assert!(!prompt1.contains("User 3 is learning Rust"));

        assert!(!prompt2.contains("User 1 prefers concise answers"));
        assert!(prompt2.contains("User 2 prefers detailed explanations"));
        assert!(!prompt2.contains("User 3 is learning Rust"));

        assert!(!prompt3.contains("User 1 prefers concise answers"));
        assert!(!prompt3.contains("User 2 prefers detailed explanations"));
        assert!(prompt3.contains("User 3 is learning Rust"));
    }
}
