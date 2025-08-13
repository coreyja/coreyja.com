use chrono::{DateTime, Utc};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::equipment::RecipeEquipment;
use super::ingredients::RecipeIngredient;
use super::steps::RecipeStep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub recipe_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub prep_time: Option<i32>, // minutes
    pub cook_time: Option<i32>, // minutes
    pub servings: i32,
    pub yield_amount: Option<f64>,
    pub yield_unit: Option<String>,
    pub author_user_id: Uuid,
    pub generated_by_stitch: Option<Uuid>,
    pub inspired_by_recipe_id: Option<Uuid>,
    pub forked_from_recipe_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Recipe {
    pub async fn create(
        pool: &PgPool,
        name: String,
        description: Option<String>,
        author_user_id: Uuid,
        prep_time: Option<i32>,
        cook_time: Option<i32>,
        servings: i32,
        yield_amount: Option<f64>,
        yield_unit: Option<String>,
        generated_by_stitch: Option<Uuid>,
        inspired_by_recipe_id: Option<Uuid>,
        forked_from_recipe_id: Option<Uuid>,
    ) -> Result<Self> {
        let recipe = sqlx::query_as!(
            Recipe,
            r#"
            INSERT INTO recipes (
                name, description, author_user_id, prep_time,
                cook_time, servings, yield_amount, yield_unit,
                generated_by_stitch, inspired_by_recipe_id, forked_from_recipe_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING 
                recipe_id,
                name,
                description,
                prep_time,
                cook_time,
                servings,
                yield_amount,
                yield_unit,
                author_user_id,
                generated_by_stitch,
                inspired_by_recipe_id,
                forked_from_recipe_id,
                created_at,
                updated_at
            "#,
            name,
            description,
            author_user_id,
            prep_time,
            cook_time,
            servings,
            yield_amount,
            yield_unit,
            generated_by_stitch,
            inspired_by_recipe_id,
            forked_from_recipe_id
        )
        .fetch_one(pool)
        .await?;

        Ok(recipe)
    }

    pub async fn get_by_id(pool: &PgPool, recipe_id: Uuid) -> Result<Option<Self>> {
        let recipe = sqlx::query_as!(
            Recipe,
            r#"
            SELECT 
                recipe_id,
                name,
                description,
                prep_time,
                cook_time,
                servings,
                yield_amount,
                yield_unit,
                author_user_id,
                generated_by_stitch,
                inspired_by_recipe_id,
                forked_from_recipe_id,
                created_at,
                updated_at
            FROM recipes
            WHERE recipe_id = $1
            "#,
            recipe_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(recipe)
    }

    pub async fn list_by_author(pool: &PgPool, author_user_id: Uuid) -> Result<Vec<Self>> {
        let recipes = sqlx::query_as!(
            Recipe,
            r#"
            SELECT 
                recipe_id,
                name,
                description,
                prep_time,
                cook_time,
                servings,
                yield_amount,
                yield_unit,
                author_user_id,
                generated_by_stitch,
                inspired_by_recipe_id,
                forked_from_recipe_id,
                created_at,
                updated_at
            FROM recipes
            WHERE author_user_id = $1
            ORDER BY created_at DESC
            "#,
            author_user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(recipes)
    }

    pub async fn update(
        &self,
        pool: &PgPool,
        name: String,
        description: Option<String>,
        prep_time: Option<i32>,
        cook_time: Option<i32>,
        servings: i32,
        yield_amount: Option<f64>,
        yield_unit: Option<String>,
    ) -> Result<Self> {
        let updated = sqlx::query_as!(
            Recipe,
            r#"
            UPDATE recipes
            SET name = $2,
                description = $3,
                prep_time = $4,
                cook_time = $5,
                servings = $6,
                yield_amount = $7,
                yield_unit = $8,
                updated_at = NOW()
            WHERE recipe_id = $1
            RETURNING 
                recipe_id,
                name,
                description,
                prep_time,
                cook_time,
                servings,
                yield_amount,
                yield_unit,
                author_user_id,
                generated_by_stitch,
                inspired_by_recipe_id,
                forked_from_recipe_id,
                created_at,
                updated_at
            "#,
            self.recipe_id,
            name,
            description,
            prep_time,
            cook_time,
            servings,
            yield_amount,
            yield_unit
        )
        .fetch_one(pool)
        .await?;

        Ok(updated)
    }

    pub async fn get_full(pool: &PgPool, recipe_id: Uuid) -> Result<Option<RecipeWithDetails>> {
        let recipe = Self::get_by_id(pool, recipe_id).await?;

        if let Some(recipe) = recipe {
            let ingredients = RecipeIngredient::get_by_recipe(pool, recipe_id).await?;
            let steps = RecipeStep::get_by_recipe(pool, recipe_id).await?;
            let equipment = RecipeEquipment::get_by_recipe(pool, recipe_id).await?;

            Ok(Some(RecipeWithDetails {
                recipe,
                ingredients,
                steps,
                equipment,
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeWithDetails {
    pub recipe: Recipe,
    pub ingredients: Vec<RecipeIngredient>,
    pub steps: Vec<RecipeStep>,
    pub equipment: Vec<RecipeEquipment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeVariation {
    pub recipe_id: Uuid,
    pub variation_id: Uuid,
    pub variation_notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl RecipeVariation {
    pub async fn create(
        pool: &PgPool,
        recipe_id: Uuid,
        variation_id: Uuid,
        variation_notes: Option<String>,
    ) -> Result<Self> {
        let variation = sqlx::query_as!(
            RecipeVariation,
            r#"
            INSERT INTO recipe_variations (
                recipe_id, variation_id, variation_notes
            )
            VALUES ($1, $2, $3)
            RETURNING 
                recipe_id,
                variation_id,
                variation_notes,
                created_at
            "#,
            recipe_id,
            variation_id,
            variation_notes
        )
        .fetch_one(pool)
        .await?;

        Ok(variation)
    }

    pub async fn get_by_recipe(pool: &PgPool, recipe_id: Uuid) -> Result<Vec<Self>> {
        let variations = sqlx::query_as!(
            RecipeVariation,
            r#"
            SELECT 
                recipe_id,
                variation_id,
                variation_notes,
                created_at
            FROM recipe_variations
            WHERE recipe_id = $1
            "#,
            recipe_id
        )
        .fetch_all(pool)
        .await?;

        Ok(variations)
    }

    pub async fn get_by_variation(pool: &PgPool, variation_id: Uuid) -> Result<Vec<Self>> {
        let variations = sqlx::query_as!(
            RecipeVariation,
            r#"
            SELECT 
                recipe_id,
                variation_id,
                variation_notes,
                created_at
            FROM recipe_variations
            WHERE variation_id = $1
            "#,
            variation_id
        )
        .fetch_all(pool)
        .await?;

        Ok(variations)
    }
}
