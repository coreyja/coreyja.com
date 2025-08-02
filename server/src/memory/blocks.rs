use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryBlockType {
    Persona,
}

impl MemoryBlockType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryBlockType::Persona => "persona",
        }
    }
}

impl std::fmt::Display for MemoryBlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for MemoryBlockType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "persona" => Ok(MemoryBlockType::Persona),
            _ => Err(format!("Unknown memory block type: {s}")),
        }
    }
}

impl TryFrom<String> for MemoryBlockType {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBlock {
    pub memory_block_id: Uuid,
    pub block_type: MemoryBlockType,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MemoryBlock {
    pub async fn create(
        pool: &PgPool,
        block_type: MemoryBlockType,
        content: String,
    ) -> color_eyre::Result<Self> {
        let row = sqlx::query!(
            r#"
            INSERT INTO memory_blocks (block_type, content)
            VALUES ($1, $2)
            RETURNING memory_block_id, block_type, content, created_at, updated_at
            "#,
            block_type.as_str(),
            content
        )
        .fetch_one(pool)
        .await?;

        Ok(MemoryBlock {
            memory_block_id: row.memory_block_id,
            block_type: row
                .block_type
                .parse::<MemoryBlockType>()
                .map_err(|e| color_eyre::eyre::eyre!(e))?,
            content: row.content,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    pub async fn find_by_id(
        pool: &PgPool,
        memory_block_id: Uuid,
    ) -> color_eyre::Result<Option<Self>> {
        let row = sqlx::query!(
            r#"
            SELECT memory_block_id, block_type, content, created_at, updated_at
            FROM memory_blocks
            WHERE memory_block_id = $1
            "#,
            memory_block_id
        )
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => Ok(Some(MemoryBlock {
                memory_block_id: row.memory_block_id,
                block_type: row
                    .block_type
                    .parse::<MemoryBlockType>()
                    .map_err(|e| color_eyre::eyre::eyre!(e))?,
                content: row.content,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })),
            None => Ok(None),
        }
    }

    pub async fn find_by_type(
        pool: &PgPool,
        block_type: MemoryBlockType,
    ) -> color_eyre::Result<Option<Self>> {
        let row = sqlx::query!(
            r#"
            SELECT memory_block_id, block_type, content, created_at, updated_at
            FROM memory_blocks
            WHERE block_type = $1
            "#,
            block_type.as_str()
        )
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => Ok(Some(MemoryBlock {
                memory_block_id: row.memory_block_id,
                block_type: row
                    .block_type
                    .parse::<MemoryBlockType>()
                    .map_err(|e| color_eyre::eyre::eyre!(e))?,
                content: row.content,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })),
            None => Ok(None),
        }
    }

    pub async fn update_content(
        pool: &PgPool,
        memory_block_id: Uuid,
        content: String,
    ) -> color_eyre::Result<Option<Self>> {
        let row = sqlx::query!(
            r#"
            UPDATE memory_blocks
            SET content = $1, updated_at = NOW()
            WHERE memory_block_id = $2
            RETURNING memory_block_id, block_type, content, created_at, updated_at
            "#,
            content,
            memory_block_id
        )
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => Ok(Some(MemoryBlock {
                memory_block_id: row.memory_block_id,
                block_type: row
                    .block_type
                    .parse::<MemoryBlockType>()
                    .map_err(|e| color_eyre::eyre::eyre!(e))?,
                content: row.content,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })),
            None => Ok(None),
        }
    }

    pub async fn delete(pool: &PgPool, memory_block_id: Uuid) -> color_eyre::Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM memory_blocks
            WHERE memory_block_id = $1
            "#,
            memory_block_id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_persona(pool: &PgPool) -> color_eyre::Result<Option<Self>> {
        Self::find_by_type(pool, MemoryBlockType::Persona).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_block_type_as_str() {
        assert_eq!(MemoryBlockType::Persona.as_str(), "persona");
    }

    #[test]
    fn test_memory_block_type_display() {
        assert_eq!(format!("{}", MemoryBlockType::Persona), "persona");
    }

    #[test]
    fn test_memory_block_type_from_str() {
        assert_eq!(
            "persona".parse::<MemoryBlockType>().unwrap(),
            MemoryBlockType::Persona
        );

        assert!("invalid".parse::<MemoryBlockType>().is_err());
    }

    #[test]
    fn test_memory_block_type_try_from_string() {
        assert_eq!(
            MemoryBlockType::try_from("persona".to_string()).unwrap(),
            MemoryBlockType::Persona
        );

        assert!(MemoryBlockType::try_from("invalid".to_string()).is_err());
    }

    #[test]
    fn test_memory_block_type_serde() {
        // Test serialization
        let block_type = MemoryBlockType::Persona;
        let json = serde_json::to_string(&block_type).unwrap();
        assert_eq!(json, r#""persona""#);

        // Test deserialization
        let deserialized: MemoryBlockType = serde_json::from_str(r#""persona""#).unwrap();
        assert_eq!(deserialized, MemoryBlockType::Persona);
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_memory_block_create(pool: PgPool) {
        let content = "Test persona content";
        let block = MemoryBlock::create(&pool, MemoryBlockType::Persona, content.to_string())
            .await
            .unwrap();

        assert_eq!(block.block_type, MemoryBlockType::Persona);
        assert_eq!(block.content, content);
        assert!(!block.memory_block_id.is_nil());
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_memory_block_find_by_id(pool: PgPool) {
        // Create a block first
        let content = "Test content for find by id";
        let created_block =
            MemoryBlock::create(&pool, MemoryBlockType::Persona, content.to_string())
                .await
                .unwrap();

        // Find it by ID
        let found_block = MemoryBlock::find_by_id(&pool, created_block.memory_block_id)
            .await
            .unwrap()
            .expect("Block should be found");

        assert_eq!(found_block.memory_block_id, created_block.memory_block_id);
        assert_eq!(found_block.content, content);
        assert_eq!(found_block.block_type, MemoryBlockType::Persona);
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_memory_block_find_by_id_not_found(pool: PgPool) {
        let random_id = Uuid::new_v4();
        let result = MemoryBlock::find_by_id(&pool, random_id).await.unwrap();

        assert!(result.is_none());
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_memory_block_find_by_type(pool: PgPool) {
        // Initially no block exists
        let found = MemoryBlock::find_by_type(&pool, MemoryBlockType::Persona)
            .await
            .unwrap();
        assert!(found.is_none());

        // Create a single persona block (due to unique constraint)
        let content = "Test persona";

        let block = MemoryBlock::create(&pool, MemoryBlockType::Persona, content.to_string())
            .await
            .unwrap();

        // Find by type
        let found = MemoryBlock::find_by_type(&pool, MemoryBlockType::Persona)
            .await
            .unwrap()
            .expect("Should find the block");

        assert_eq!(found.memory_block_id, block.memory_block_id);
        assert_eq!(found.content, content);
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_memory_block_update_content(pool: PgPool) {
        // Create a block
        let initial_content = "Initial content";
        let block =
            MemoryBlock::create(&pool, MemoryBlockType::Persona, initial_content.to_string())
                .await
                .unwrap();

        // Update its content
        let new_content = "Updated content";
        let updated_block =
            MemoryBlock::update_content(&pool, block.memory_block_id, new_content.to_string())
                .await
                .unwrap()
                .expect("Update should succeed");

        assert_eq!(updated_block.content, new_content);
        assert_eq!(updated_block.memory_block_id, block.memory_block_id);
        assert!(updated_block.updated_at > block.updated_at);
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_memory_block_delete(pool: PgPool) {
        // Create a block
        let block =
            MemoryBlock::create(&pool, MemoryBlockType::Persona, "To be deleted".to_string())
                .await
                .unwrap();

        // Delete it
        let deleted = MemoryBlock::delete(&pool, block.memory_block_id)
            .await
            .unwrap();

        assert!(deleted);

        // Verify it's gone
        let found = MemoryBlock::find_by_id(&pool, block.memory_block_id)
            .await
            .unwrap();

        assert!(found.is_none());
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_memory_block_get_persona(pool: PgPool) {
        // Initially no persona
        let persona = MemoryBlock::get_persona(&pool).await.unwrap();
        assert!(persona.is_none());

        // Create a persona
        let content = "I am a helpful AI assistant";
        let created = MemoryBlock::create(&pool, MemoryBlockType::Persona, content.to_string())
            .await
            .unwrap();

        // Get persona should return it
        let persona = MemoryBlock::get_persona(&pool)
            .await
            .unwrap()
            .expect("Persona should exist");

        assert_eq!(persona.memory_block_id, created.memory_block_id);
        assert_eq!(persona.content, content);
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_memory_block_persona_unique_constraint(pool: PgPool) {
        // Create first persona
        let _first =
            MemoryBlock::create(&pool, MemoryBlockType::Persona, "First persona".to_string())
                .await
                .unwrap();

        // Try to create second persona - should fail due to unique constraint
        let result = MemoryBlock::create(
            &pool,
            MemoryBlockType::Persona,
            "Second persona".to_string(),
        )
        .await;

        assert!(result.is_err());
    }
}
