use color_eyre::Result;
use sqlx::PgPool;

use super::blocks::{MemoryBlock, MemoryBlockType};

#[derive(Debug)]
pub struct PersonaManager;

impl PersonaManager {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_current_persona(pool: &PgPool) -> Result<Option<String>> {
        let persona = MemoryBlock::get_persona(pool).await?;
        Ok(persona.map(|p| p.content))
    }

    pub async fn update_persona(pool: &PgPool, content: String) -> Result<MemoryBlock> {
        // First try to find existing persona
        let existing = MemoryBlock::find_by_type(pool, MemoryBlockType::Persona).await?;

        if let Some(persona) = existing {
            // Update existing persona
            MemoryBlock::update_content(pool, persona.memory_block_id, content)
                .await?
                .ok_or_else(|| color_eyre::eyre::eyre!("Failed to update persona"))
        } else {
            // Create new persona
            MemoryBlock::create(pool, MemoryBlockType::Persona, content).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_manager_new() {
        let manager = PersonaManager::new();
        // Just verify it can be created
        let _ = format!("{manager:?}");
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_get_current_persona_none(pool: PgPool) {
        let persona = PersonaManager::get_current_persona(&pool).await.unwrap();
        assert!(persona.is_none());
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_get_current_persona_exists(pool: PgPool) {
        // Create a persona
        let content = "I am a helpful AI assistant with a friendly personality";
        MemoryBlock::create(&pool, MemoryBlockType::Persona, content.to_string())
            .await
            .unwrap();

        let persona = PersonaManager::get_current_persona(&pool).await.unwrap();
        assert_eq!(persona, Some(content.to_string()));
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_update_persona_creates_new(pool: PgPool) {
        let content = "Brand new persona";
        let created = PersonaManager::update_persona(&pool, content.to_string())
            .await
            .unwrap();

        assert_eq!(created.content, content);
        assert_eq!(created.block_type, MemoryBlockType::Persona);
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_update_persona_updates_existing(pool: PgPool) {
        // Create initial persona
        let initial_content = "Initial persona";
        let initial = PersonaManager::update_persona(&pool, initial_content.to_string())
            .await
            .unwrap();

        // Update it
        let updated_content = "Updated persona";
        let updated = PersonaManager::update_persona(&pool, updated_content.to_string())
            .await
            .unwrap();

        assert_eq!(updated.memory_block_id, initial.memory_block_id);
        assert_eq!(updated.content, updated_content);
        assert!(updated.updated_at > initial.updated_at);
    }
}
