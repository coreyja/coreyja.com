use color_eyre::Result;
use sqlx::PgPool;

use super::blocks::{MemoryBlock, MemoryBlockType};
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
        // First try to find existing persona
        let existing = MemoryBlock::find_by_type(&self.pool, MemoryBlockType::Persona).await?;

        if let Some(persona) = existing.first() {
            // Update existing persona
            MemoryBlock::update_content(&self.pool, persona.memory_block_id, content)
                .await?
                .ok_or_else(|| color_eyre::eyre::eyre!("Failed to update persona"))
        } else {
            // Create new persona
            MemoryBlock::create(&self.pool, MemoryBlockType::Persona, content).await
        }
    }

    /// Build complete system prompt with persona
    pub async fn generate_system_prompt(&self, is_discord_context: bool) -> Result<String> {
        PromptGenerator::generate_system_prompt(&self.pool, is_discord_context).await
    }

    /// Get reference to the database pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        MemoryBlock::create(&pool, MemoryBlockType::Persona, content.to_string())
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

        // Test without persona, non-Discord
        let prompt = manager.generate_system_prompt(false).await.unwrap();
        assert!(prompt.contains("AI assistant"));
        assert!(!prompt.contains("Discord"));

        // Test with Discord context
        let discord_prompt = manager.generate_system_prompt(true).await.unwrap();
        assert!(discord_prompt.contains("AI assistant"));
        assert!(discord_prompt.contains("Discord"));

        // Add persona and test again
        let persona_content = "I am a test persona for the manager";
        manager
            .update_persona(persona_content.to_string())
            .await
            .unwrap();

        let prompt_with_persona = manager.generate_system_prompt(false).await.unwrap();
        assert!(prompt_with_persona.contains(persona_content));
    }
}
