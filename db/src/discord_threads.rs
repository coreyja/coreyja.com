use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DiscordThreadMetadata {
    pub thread_id: Uuid,
    pub discord_thread_id: String,
    pub channel_id: String,
    pub guild_id: String,
    pub last_message_id: Option<String>,
    pub created_by: String,
    pub thread_name: String,
    pub participants: JsonValue, // JSON array of participant tags
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DiscordThreadMetadata {
    pub async fn create(
        pool: &PgPool,
        thread_id: Uuid,
        discord_thread_id: String,
        channel_id: String,
        guild_id: String,
        created_by: String,
        thread_name: String,
    ) -> color_eyre::Result<Self> {
        let metadata = sqlx::query_as!(
            DiscordThreadMetadata,
            r#"
            INSERT INTO discord_thread_metadata 
                (thread_id, discord_thread_id, channel_id, guild_id, created_by, thread_name)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            thread_id,
            discord_thread_id,
            channel_id,
            guild_id,
            created_by,
            thread_name
        )
        .fetch_one(pool)
        .await?;

        Ok(metadata)
    }

    pub async fn find_by_discord_thread_id(
        pool: &PgPool,
        discord_thread_id: &str,
    ) -> color_eyre::Result<Option<Self>> {
        let metadata = sqlx::query_as!(
            DiscordThreadMetadata,
            r#"
            SELECT * FROM discord_thread_metadata
            WHERE discord_thread_id = $1
            "#,
            discord_thread_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(metadata)
    }

    pub async fn find_by_thread_id(
        pool: &PgPool,
        thread_id: Uuid,
    ) -> color_eyre::Result<Option<Self>> {
        let metadata = sqlx::query_as!(
            DiscordThreadMetadata,
            r#"
            SELECT * FROM discord_thread_metadata
            WHERE thread_id = $1
            "#,
            thread_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(metadata)
    }

    pub async fn update_last_message_id(
        pool: &PgPool,
        thread_id: Uuid,
        last_message_id: String,
    ) -> color_eyre::Result<Option<Self>> {
        let metadata = sqlx::query_as!(
            DiscordThreadMetadata,
            r#"
            UPDATE discord_thread_metadata
            SET last_message_id = $1, updated_at = NOW()
            WHERE thread_id = $2
            RETURNING *
            "#,
            last_message_id,
            thread_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(metadata)
    }

    pub async fn add_participant(
        pool: &PgPool,
        thread_id: Uuid,
        participant_tag: &str,
    ) -> color_eyre::Result<Option<Self>> {
        let metadata = sqlx::query_as!(
            DiscordThreadMetadata,
            r#"
            UPDATE discord_thread_metadata
            SET participants = 
                CASE 
                    WHEN participants ? $1 THEN participants
                    ELSE participants || to_jsonb($1::text)
                END,
                updated_at = NOW()
            WHERE thread_id = $2
            RETURNING *
            "#,
            participant_tag,
            thread_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(metadata)
    }
}
