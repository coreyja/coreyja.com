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
    ) -> color_eyre::Result<Vec<Self>> {
        let rows = sqlx::query!(
            r#"
            SELECT memory_block_id, block_type, content, created_at, updated_at
            FROM memory_blocks
            WHERE block_type = $1
            ORDER BY created_at DESC
            "#,
            block_type.as_str()
        )
        .fetch_all(pool)
        .await?;

        rows.into_iter()
            .map(|row| {
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
            })
            .collect()
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
        Self::find_by_type(pool, MemoryBlockType::Persona)
            .await
            .map(|blocks| blocks.into_iter().next())
    }
}
