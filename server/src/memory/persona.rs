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

        if let Some(persona) = existing.first() {
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
