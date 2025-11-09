use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBlock {
    pub memory_block_id: Uuid,
    pub memory_type: String,
    pub identifier: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MemoryBlock {
    pub async fn create(
        pool: &PgPool,
        memory_type: String,
        identifier: String,
        content: String,
    ) -> color_eyre::Result<Self> {
        let block = sqlx::query_as!(
            MemoryBlock,
            r#"
            INSERT INTO memory_blocks (type, identifier, content)
            VALUES ($1, $2, $3)
            RETURNING
                memory_block_id,
                type as memory_type,
                identifier,
                content,
                created_at,
                updated_at
            "#,
            memory_type,
            identifier,
            content
        )
        .fetch_one(pool)
        .await?;

        Ok(block)
    }

    pub async fn find_by_id(
        pool: &PgPool,
        memory_block_id: Uuid,
    ) -> color_eyre::Result<Option<Self>> {
        let block = sqlx::query_as!(
            MemoryBlock,
            r#"
            SELECT
                memory_block_id,
                type as memory_type,
                identifier,
                content,
                created_at,
                updated_at
            FROM memory_blocks
            WHERE memory_block_id = $1
            "#,
            memory_block_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(block)
    }

    pub async fn find_by_type_and_identifier(
        pool: &PgPool,
        memory_type: String,
        identifier: String,
    ) -> color_eyre::Result<Option<Self>> {
        let block = sqlx::query_as!(
            MemoryBlock,
            r#"
            SELECT
                memory_block_id,
                type as memory_type,
                identifier,
                content,
                created_at,
                updated_at
            FROM memory_blocks
            WHERE type = $1 AND identifier = $2
            "#,
            memory_type,
            identifier
        )
        .fetch_optional(pool)
        .await?;

        Ok(block)
    }

    pub async fn list_by_type(pool: &PgPool, memory_type: String) -> color_eyre::Result<Vec<Self>> {
        let blocks = sqlx::query_as!(
            MemoryBlock,
            r#"
            SELECT
                memory_block_id,
                type as memory_type,
                identifier,
                content,
                created_at,
                updated_at
            FROM memory_blocks
            WHERE type = $1
            ORDER BY identifier
            "#,
            memory_type
        )
        .fetch_all(pool)
        .await?;

        Ok(blocks)
    }

    pub async fn get_all_types_with_counts(
        pool: &PgPool,
    ) -> color_eyre::Result<Vec<(String, i64)>> {
        let results = sqlx::query!(
            r#"
            SELECT type, COUNT(*) as "count!"
            FROM memory_blocks
            GROUP BY type
            ORDER BY type
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(results.into_iter().map(|r| (r.r#type, r.count)).collect())
    }

    pub async fn update_content(
        pool: &PgPool,
        memory_block_id: Uuid,
        content: String,
    ) -> color_eyre::Result<Option<Self>> {
        let block = sqlx::query_as!(
            MemoryBlock,
            r#"
            UPDATE memory_blocks
            SET content = $1, updated_at = NOW()
            WHERE memory_block_id = $2
            RETURNING
                memory_block_id,
                type as memory_type,
                identifier,
                content,
                created_at,
                updated_at
            "#,
            content,
            memory_block_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(block)
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

    pub async fn delete_by_type_and_identifier(
        pool: &PgPool,
        memory_type: String,
        identifier: String,
    ) -> color_eyre::Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM memory_blocks
            WHERE type = $1 AND identifier = $2
            "#,
            memory_type,
            identifier
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_persona(pool: &PgPool) -> color_eyre::Result<Option<Self>> {
        Self::find_by_type_and_identifier(pool, "persona".to_string(), "default".to_string()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // New tests for extended functionality (Task 1.1)

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_find_by_type_and_identifier(pool: PgPool) {
        // Create multiple person blocks with different identifiers
        let block1 = MemoryBlock::create(
            &pool,
            "person".to_string(),
            "corey".to_string(),
            "Information about Corey".to_string(),
        )
        .await
        .unwrap();

        let block2 = MemoryBlock::create(
            &pool,
            "person".to_string(),
            "jane".to_string(),
            "Information about Jane".to_string(),
        )
        .await
        .unwrap();

        // Find specific person block
        let found_corey = MemoryBlock::find_by_type_and_identifier(
            &pool,
            "person".to_string(),
            "corey".to_string(),
        )
        .await
        .unwrap()
        .expect("Should find Corey's block");

        assert_eq!(found_corey.memory_block_id, block1.memory_block_id);
        assert_eq!(found_corey.identifier, "corey");
        assert_eq!(found_corey.content, "Information about Corey");

        // Find different person block
        let found_jane = MemoryBlock::find_by_type_and_identifier(
            &pool,
            "person".to_string(),
            "jane".to_string(),
        )
        .await
        .unwrap()
        .expect("Should find Jane's block");

        assert_eq!(found_jane.memory_block_id, block2.memory_block_id);
        assert_eq!(found_jane.identifier, "jane");

        // Try to find non-existent block
        let not_found = MemoryBlock::find_by_type_and_identifier(
            &pool,
            "person".to_string(),
            "unknown".to_string(),
        )
        .await
        .unwrap();

        assert!(not_found.is_none());
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_list_by_type(pool: PgPool) {
        // Create multiple person blocks
        let _block1 = MemoryBlock::create(
            &pool,
            "person".to_string(),
            "corey".to_string(),
            "Information about Corey".to_string(),
        )
        .await
        .unwrap();

        let _block2 = MemoryBlock::create(
            &pool,
            "person".to_string(),
            "jane".to_string(),
            "Information about Jane".to_string(),
        )
        .await
        .unwrap();

        // Create a persona block (different type)
        let _block3 = MemoryBlock::create(
            &pool,
            "persona".to_string(),
            "default".to_string(),
            "I am a helpful AI".to_string(),
        )
        .await
        .unwrap();

        // List all person blocks
        let person_blocks = MemoryBlock::list_by_type(&pool, "person".to_string())
            .await
            .unwrap();

        assert_eq!(person_blocks.len(), 2);
        let identifiers: Vec<&str> = person_blocks
            .iter()
            .map(|b| b.identifier.as_str())
            .collect();
        assert!(identifiers.contains(&"corey"));
        assert!(identifiers.contains(&"jane"));

        // List all persona blocks
        let persona_blocks = MemoryBlock::list_by_type(&pool, "persona".to_string())
            .await
            .unwrap();

        assert_eq!(persona_blocks.len(), 1);
        assert_eq!(persona_blocks[0].identifier, "default");

        // List non-existent type
        let empty = MemoryBlock::list_by_type(&pool, "nonexistent".to_string())
            .await
            .unwrap();

        assert_eq!(empty.len(), 0);
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_unique_constraint_on_type_identifier(pool: PgPool) {
        // Create first person block with identifier "corey"
        let _first = MemoryBlock::create(
            &pool,
            "person".to_string(),
            "corey".to_string(),
            "First content".to_string(),
        )
        .await
        .unwrap();

        // Try to create another person block with same identifier - should fail
        let result = MemoryBlock::create(
            &pool,
            "person".to_string(),
            "corey".to_string(),
            "Second content".to_string(),
        )
        .await;

        assert!(result.is_err());

        // But we CAN create a different type with same identifier
        let different_type = MemoryBlock::create(
            &pool,
            "other_type".to_string(),
            "corey".to_string(),
            "Different type content".to_string(),
        )
        .await
        .unwrap();

        assert_eq!(different_type.memory_type, "other_type");
        assert_eq!(different_type.identifier, "corey");
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_get_persona_hardcoded_to_default(pool: PgPool) {
        // Create persona with identifier "default"
        let _default_persona = MemoryBlock::create(
            &pool,
            "persona".to_string(),
            "default".to_string(),
            "I am the default persona".to_string(),
        )
        .await
        .unwrap();

        // Create another persona with different identifier
        let _other_persona = MemoryBlock::create(
            &pool,
            "persona".to_string(),
            "other".to_string(),
            "I am another persona".to_string(),
        )
        .await
        .unwrap();

        // get_persona() should return only the "default" one
        let persona = MemoryBlock::get_persona(&pool)
            .await
            .unwrap()
            .expect("Should find default persona");

        assert_eq!(persona.memory_type, "persona");
        assert_eq!(persona.identifier, "default");
        assert_eq!(persona.content, "I am the default persona");
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_get_all_types_with_counts(pool: PgPool) {
        // Create multiple blocks of different types
        let _p1 = MemoryBlock::create(
            &pool,
            "person".to_string(),
            "corey".to_string(),
            "Content 1".to_string(),
        )
        .await
        .unwrap();

        let _p2 = MemoryBlock::create(
            &pool,
            "person".to_string(),
            "jane".to_string(),
            "Content 2".to_string(),
        )
        .await
        .unwrap();

        let _persona = MemoryBlock::create(
            &pool,
            "persona".to_string(),
            "default".to_string(),
            "Persona content".to_string(),
        )
        .await
        .unwrap();

        // Get all types with counts
        let types_with_counts = MemoryBlock::get_all_types_with_counts(&pool).await.unwrap();

        assert_eq!(types_with_counts.len(), 2);

        // Find counts for each type
        let person_count = types_with_counts
            .iter()
            .find(|(t, _)| t == "person")
            .map(|(_, c)| *c)
            .unwrap();
        assert_eq!(person_count, 2);

        let persona_count = types_with_counts
            .iter()
            .find(|(t, _)| t == "persona")
            .map(|(_, c)| *c)
            .unwrap();
        assert_eq!(persona_count, 1);
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_delete_by_type_and_identifier(pool: PgPool) {
        // Create a block
        let _block = MemoryBlock::create(
            &pool,
            "person".to_string(),
            "corey".to_string(),
            "To be deleted".to_string(),
        )
        .await
        .unwrap();

        // Verify it exists
        let found = MemoryBlock::find_by_type_and_identifier(
            &pool,
            "person".to_string(),
            "corey".to_string(),
        )
        .await
        .unwrap();
        assert!(found.is_some());

        // Delete it
        let deleted = MemoryBlock::delete_by_type_and_identifier(
            &pool,
            "person".to_string(),
            "corey".to_string(),
        )
        .await
        .unwrap();

        assert!(deleted);

        // Verify it's gone
        let not_found = MemoryBlock::find_by_type_and_identifier(
            &pool,
            "person".to_string(),
            "corey".to_string(),
        )
        .await
        .unwrap();

        assert!(not_found.is_none());

        // Try to delete non-existent block - should return false
        let not_deleted = MemoryBlock::delete_by_type_and_identifier(
            &pool,
            "person".to_string(),
            "nonexistent".to_string(),
        )
        .await
        .unwrap();

        assert!(!not_deleted);
    }

    // Task Group 5 Integration Tests (Strategic gap-filling tests)

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_multiple_memory_types_coexist_correctly(pool: PgPool) {
        // Integration test: Create multiple types with overlapping identifiers
        // This tests that the (type, identifier) unique constraint works correctly

        // Create persona
        let persona = MemoryBlock::create(
            &pool,
            "persona".to_string(),
            "default".to_string(),
            "I am a helpful AI assistant.".to_string(),
        )
        .await
        .unwrap();

        // Create person blocks with various identifiers
        let person1 = MemoryBlock::create(
            &pool,
            "person".to_string(),
            "alice#1234".to_string(),
            "Alice is a software engineer.".to_string(),
        )
        .await
        .unwrap();

        let person2 = MemoryBlock::create(
            &pool,
            "person".to_string(),
            "bob#5678".to_string(),
            "Bob is a product manager.".to_string(),
        )
        .await
        .unwrap();

        // Create a custom type with identifier "default" (same identifier as persona, different type)
        let custom = MemoryBlock::create(
            &pool,
            "custom".to_string(),
            "default".to_string(),
            "This is a custom memory type.".to_string(),
        )
        .await
        .unwrap();

        // Verify all blocks are distinct
        assert_ne!(persona.memory_block_id, person1.memory_block_id);
        assert_ne!(persona.memory_block_id, person2.memory_block_id);
        assert_ne!(persona.memory_block_id, custom.memory_block_id);
        assert_ne!(person1.memory_block_id, person2.memory_block_id);

        // Verify we can retrieve each block by type-identifier pair
        let retrieved_persona = MemoryBlock::find_by_type_and_identifier(
            &pool,
            "persona".to_string(),
            "default".to_string(),
        )
        .await
        .unwrap()
        .expect("Should find persona");
        assert_eq!(retrieved_persona.memory_block_id, persona.memory_block_id);

        let retrieved_custom = MemoryBlock::find_by_type_and_identifier(
            &pool,
            "custom".to_string(),
            "default".to_string(),
        )
        .await
        .unwrap()
        .expect("Should find custom");
        assert_eq!(retrieved_custom.memory_block_id, custom.memory_block_id);

        // Verify type counts
        let types_with_counts = MemoryBlock::get_all_types_with_counts(&pool).await.unwrap();
        assert_eq!(types_with_counts.len(), 3); // persona, person, custom

        let person_count = types_with_counts
            .iter()
            .find(|(t, _)| t == "person")
            .map(|(_, c)| *c)
            .unwrap();
        assert_eq!(person_count, 2);

        let persona_count = types_with_counts
            .iter()
            .find(|(t, _)| t == "persona")
            .map(|(_, c)| *c)
            .unwrap();
        assert_eq!(persona_count, 1);

        let custom_count = types_with_counts
            .iter()
            .find(|(t, _)| t == "custom")
            .map(|(_, c)| *c)
            .unwrap();
        assert_eq!(custom_count, 1);
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_unique_constraint_violation_error_details(pool: PgPool) {
        // Create a memory block
        let _original = MemoryBlock::create(
            &pool,
            "person".to_string(),
            "testuser#1111".to_string(),
            "Original content".to_string(),
        )
        .await
        .unwrap();

        // Attempt to create duplicate (same type and identifier)
        let duplicate_result = MemoryBlock::create(
            &pool,
            "person".to_string(),
            "testuser#1111".to_string(),
            "Duplicate content".to_string(),
        )
        .await;

        // Verify it fails
        assert!(duplicate_result.is_err());

        // Verify the error is a database constraint violation
        let err = duplicate_result.unwrap_err();
        let err_string = format!("{err:?}");

        // Should mention unique constraint or duplicate key
        assert!(
            err_string.contains("unique")
                || err_string.contains("duplicate")
                || err_string.contains("constraint"),
            "Error should mention constraint violation: {err_string}"
        );
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_memory_block_lifecycle_complete_flow(pool: PgPool) {
        // Integration test: Full lifecycle of a memory block

        // 1. Create
        let created = MemoryBlock::create(
            &pool,
            "person".to_string(),
            "lifecycle#0001".to_string(),
            "Initial content".to_string(),
        )
        .await
        .unwrap();

        assert_eq!(created.memory_type, "person");
        assert_eq!(created.identifier, "lifecycle#0001");
        assert_eq!(created.content, "Initial content");

        // 2. Find by type and identifier
        let found = MemoryBlock::find_by_type_and_identifier(
            &pool,
            "person".to_string(),
            "lifecycle#0001".to_string(),
        )
        .await
        .unwrap()
        .expect("Should find the created block");
        assert_eq!(found.memory_block_id, created.memory_block_id);

        // 3. Find by ID
        let found_by_id = MemoryBlock::find_by_id(&pool, created.memory_block_id)
            .await
            .unwrap()
            .expect("Should find by ID");
        assert_eq!(found_by_id.memory_block_id, created.memory_block_id);

        // 4. Update content
        let updated = MemoryBlock::update_content(
            &pool,
            created.memory_block_id,
            "Updated content".to_string(),
        )
        .await
        .unwrap()
        .expect("Should update successfully");
        assert_eq!(updated.content, "Updated content");
        assert_eq!(updated.memory_block_id, created.memory_block_id);
        assert!(updated.updated_at > created.created_at);

        // 5. Verify it appears in type listing
        let person_blocks = MemoryBlock::list_by_type(&pool, "person".to_string())
            .await
            .unwrap();
        assert!(person_blocks
            .iter()
            .any(|b| b.memory_block_id == created.memory_block_id));

        // 6. Verify it appears in type counts
        let types_with_counts = MemoryBlock::get_all_types_with_counts(&pool).await.unwrap();
        let person_count = types_with_counts
            .iter()
            .find(|(t, _)| t == "person")
            .map(|(_, c)| *c)
            .unwrap();
        assert!(person_count >= 1);

        // 7. Delete
        let deleted = MemoryBlock::delete_by_type_and_identifier(
            &pool,
            "person".to_string(),
            "lifecycle#0001".to_string(),
        )
        .await
        .unwrap();
        assert!(deleted);

        // 8. Verify deletion
        let not_found = MemoryBlock::find_by_type_and_identifier(
            &pool,
            "person".to_string(),
            "lifecycle#0001".to_string(),
        )
        .await
        .unwrap();
        assert!(not_found.is_none());

        let not_found_by_id = MemoryBlock::find_by_id(&pool, created.memory_block_id)
            .await
            .unwrap();
        assert!(not_found_by_id.is_none());
    }
}
