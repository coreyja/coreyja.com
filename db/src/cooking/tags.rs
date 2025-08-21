use chrono::{DateTime, Utc};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub tag_id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Tag {
    pub async fn create(pool: &PgPool, name: String, color: Option<String>) -> Result<Self> {
        let tag = sqlx::query_as!(
            Tag,
            r#"
            INSERT INTO tags (name, color)
            VALUES ($1, $2)
            RETURNING 
                tag_id,
                name,
                color,
                created_at,
                updated_at
            "#,
            name,
            color
        )
        .fetch_one(pool)
        .await?;

        Ok(tag)
    }

    pub async fn get_by_id(pool: &PgPool, tag_id: Uuid) -> Result<Option<Self>> {
        let tag = sqlx::query_as!(
            Tag,
            r#"
            SELECT 
                tag_id,
                name,
                color,
                created_at,
                updated_at
            FROM tags
            WHERE tag_id = $1
            "#,
            tag_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(tag)
    }

    pub async fn get_by_name(pool: &PgPool, name: &str) -> Result<Option<Self>> {
        let tag = sqlx::query_as!(
            Tag,
            r#"
            SELECT 
                tag_id,
                name,
                color,
                created_at,
                updated_at
            FROM tags
            WHERE name = $1
            "#,
            name
        )
        .fetch_optional(pool)
        .await?;

        Ok(tag)
    }

    pub async fn list_all(pool: &PgPool) -> Result<Vec<Self>> {
        let tags = sqlx::query_as!(
            Tag,
            r#"
            SELECT 
                tag_id,
                name,
                color,
                created_at,
                updated_at
            FROM tags
            ORDER BY name
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(tags)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeTag {
    pub recipe_id: Uuid,
    pub tag_id: Uuid,
}

impl RecipeTag {
    pub async fn create(pool: &PgPool, recipe_id: Uuid, tag_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO recipe_tags (recipe_id, tag_id)
            VALUES ($1, $2)
            "#,
            recipe_id,
            tag_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_by_recipe(pool: &PgPool, recipe_id: Uuid) -> Result<Vec<Tag>> {
        let tags = sqlx::query_as!(
            Tag,
            r#"
            SELECT 
                t.tag_id,
                t.name,
                t.color,
                t.created_at,
                t.updated_at
            FROM tags t
            JOIN recipe_tags rt ON t.tag_id = rt.tag_id
            WHERE rt.recipe_id = $1
            ORDER BY t.name
            "#,
            recipe_id
        )
        .fetch_all(pool)
        .await?;

        Ok(tags)
    }

    pub async fn get_recipes_by_tag(pool: &PgPool, tag_id: Uuid) -> Result<Vec<Uuid>> {
        let recipe_ids = sqlx::query!(
            r#"
            SELECT recipe_id
            FROM recipe_tags
            WHERE tag_id = $1
            "#,
            tag_id
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| row.recipe_id)
        .collect();

        Ok(recipe_ids)
    }

    pub async fn delete(pool: &PgPool, recipe_id: Uuid, tag_id: Uuid) -> Result<()> {
        sqlx::query!(
            "DELETE FROM recipe_tags WHERE recipe_id = $1 AND tag_id = $2",
            recipe_id,
            tag_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn set_tags_for_recipe(
        pool: &PgPool,
        recipe_id: Uuid,
        tag_ids: Vec<Uuid>,
    ) -> Result<()> {
        let mut transaction = pool.begin().await?;

        sqlx::query!("DELETE FROM recipe_tags WHERE recipe_id = $1", recipe_id)
            .execute(&mut *transaction)
            .await?;

        for tag_id in tag_ids {
            sqlx::query!(
                "INSERT INTO recipe_tags (recipe_id, tag_id) VALUES ($1, $2)",
                recipe_id,
                tag_id
            )
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }
}
