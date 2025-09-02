use std::str::FromStr;

use db::cooking::Recipe;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    al::tools::{ThreadContext, Tool},
    AppState,
};

// UpsertRecipe Tool
#[derive(Clone, Debug)]
pub struct UpsertRecipe;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpsertRecipeInput {
    pub recipe_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub prep_time: Option<i32>,
    pub cook_time: Option<i32>,
    pub servings: i32,
    pub ingredients: Vec<RecipeIngredientInput>,
    pub steps: Vec<RecipeStepInput>,
    pub equipment: Vec<RecipeEquipmentInput>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RecipeIngredientInput {
    pub ingredient_name: String,
    pub quantity: f64,
    pub unit_name: String,
    pub is_optional: Option<bool>,
    pub notes: Option<String>,
    pub ingredient_group: Option<String>,
    pub preparation: Option<String>,
    pub temperature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RecipeStepInput {
    pub step_number: i32,
    pub instruction: String,
    pub duration: Option<i32>,
    pub temperature: Option<i32>,
    pub temperature_unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RecipeEquipmentInput {
    pub equipment_name: String,
    pub is_optional: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpsertRecipeOutput {
    pub recipe_id: String,
}

#[async_trait::async_trait]
impl Tool for UpsertRecipe {
    const NAME: &'static str = "upsert_recipe";
    const DESCRIPTION: &'static str = r#"
    Create or update a recipe with full details including steps, ingredients, and equipment.

    If recipe_id is provided, updates the existing recipe (replacing all ingredients, steps, equipment).
    If recipe_id is not provided, creates a new recipe.
    Automatically creates any missing ingredients, units, equipment, or tags.

    Example:
    ```json
    {
        "name": "Chocolate Chip Cookies",
        "description": "Classic homemade cookies",
        "prep_time": 15,
        "cook_time": 12,
        "servings": 24,
        "ingredients": [
            {
                "ingredient_name": "all-purpose flour",
                "quantity": 2.25,
                "unit_name": "cups",
                "ingredient_group": "Dry ingredients"
            },
            {
                "ingredient_name": "chocolate chips",
                "quantity": 2,
                "unit_name": "cups",
                "is_optional": false
            }
        ],
        "steps": [
            {
                "step_number": 1,
                "instruction": "Preheat oven to 375°F",
                "temperature": 375,
                "temperature_unit": "F"
            },
            {
                "step_number": 2,
                "instruction": "Mix dry ingredients in a bowl",
                "duration": 5
            }
        ],
        "equipment": [
            {
                "equipment_name": "mixing bowl",
                "is_optional": false
            },
            {
                "equipment_name": "electric mixer",
                "is_optional": true,
                "notes": "Can mix by hand instead"
            }
        ],
        "tags": ["dessert", "cookies", "baking"]
    }
    ```
    "#;

    type ToolInput = UpsertRecipeInput;
    type ToolOutput = UpsertRecipeOutput;

    #[allow(clippy::too_many_lines)]
    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        let pool = &app_state.db;

        // Parse UUIDs
        let recipe_id = input
            .recipe_id
            .as_ref()
            .map(|id| Uuid::parse_str(id))
            .transpose()?;

        // Create or update the recipe
        let recipe = if let Some(recipe_id) = recipe_id {
            // Update existing recipe
            sqlx::query_as!(
                Recipe,
                r#"
                UPDATE recipes
                SET name = $2,
                    description = $3,
                    prep_time = $4,
                    cook_time = $5,
                    servings = $6,
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
                    generated_by_stitch,
                    inspired_by_recipe_id,
                    forked_from_recipe_id,
                    created_at,
                    updated_at
                "#,
                recipe_id,
                input.name,
                input.description,
                input.prep_time,
                input.cook_time,
                input.servings
            )
            .fetch_one(pool)
            .await?
        } else {
            // Create new recipe
            Recipe::create(
                pool,
                input.name,
                input.description,
                input.prep_time,
                input.cook_time,
                input.servings,
                None, // yield_amount
                None, // yield_unit
                Some(
                    context
                        .previous_stitch_id
                        .unwrap_or(context.thread.thread_id),
                ),
                None, // inspired_by_recipe_id
                None, // forked_from_recipe_id
            )
            .await?
        };

        // If updating, clear existing ingredients, steps, equipment, and tags
        if input.recipe_id.is_some() {
            // Delete existing recipe ingredients
            sqlx::query!(
                "DELETE FROM recipe_ingredients WHERE recipe_id = $1",
                recipe.recipe_id
            )
            .execute(pool)
            .await?;

            // Delete existing recipe steps (this will cascade to step_ingredients and step_equipment)
            sqlx::query!(
                "DELETE FROM recipe_steps WHERE recipe_id = $1",
                recipe.recipe_id
            )
            .execute(pool)
            .await?;

            // Delete existing recipe equipment
            sqlx::query!(
                "DELETE FROM recipe_equipment WHERE recipe_id = $1",
                recipe.recipe_id
            )
            .execute(pool)
            .await?;

            // Delete existing recipe tags
            sqlx::query!(
                "DELETE FROM recipe_tags WHERE recipe_id = $1",
                recipe.recipe_id
            )
            .execute(pool)
            .await?;
        }

        // Add ingredients
        for ingredient_input in &input.ingredients {
            // Get or create ingredient
            let ingredient =
                match db::cooking::Ingredient::get_by_name(pool, &ingredient_input.ingredient_name)
                    .await?
                {
                    Some(i) => i,
                    None => {
                        db::cooking::Ingredient::create(
                            pool,
                            ingredient_input.ingredient_name.clone(),
                            None,
                            None,
                        )
                        .await?
                    }
                };

            // Get or create unit
            let unit = match db::cooking::Unit::get_by_name(pool, &ingredient_input.unit_name)
                .await?
            {
                Some(u) => u,
                None => {
                    // Create unit with default type
                    sqlx::query_as!(
                        db::cooking::Unit,
                        r#"
                        INSERT INTO units (unit_id, name, type, created_at, updated_at)
                        VALUES ($1, $2, $3, NOW(), NOW())
                        RETURNING unit_id, name, type as "unit_type: db::cooking::UnitType", created_at, updated_at
                        "#,
                        Uuid::new_v4(),
                        ingredient_input.unit_name,
                        Some("volume")
                    )
                    .fetch_one(pool)
                    .await?
                }
            };

            // Convert f64 to BigDecimal
            let quantity =
                sqlx::types::BigDecimal::from_str(&ingredient_input.quantity.to_string())?;

            // Create recipe ingredient
            db::cooking::RecipeIngredient::create(
                pool,
                recipe.recipe_id,
                ingredient.ingredient_id,
                quantity,
                unit.unit_id,
                None, // display_order
                ingredient_input.ingredient_group.clone(),
                ingredient_input
                    .preparation
                    .as_ref()
                    .and_then(|p| db::cooking::IngredientPreparation::from_str(p).ok()),
                ingredient_input
                    .temperature
                    .as_ref()
                    .and_then(|t| db::cooking::IngredientTemperature::from_str(t).ok()),
                ingredient_input.is_optional,
                ingredient_input.notes.clone(),
            )
            .await?;
        }

        // Add steps
        let steps: Vec<db::cooking::RecipeStep> = input
            .steps
            .iter()
            .map(|step| {
                let temp_unit = step
                    .temperature_unit
                    .as_ref()
                    .and_then(|t| db::cooking::TemperatureUnit::from_str(t).ok());
                let temperature = step.temperature;

                db::cooking::RecipeStep {
                    step_id: Uuid::new_v4(),
                    recipe_id: recipe.recipe_id,
                    step_number: step.step_number,
                    instruction: step.instruction.clone(),
                    duration: step.duration,
                    temperature,
                    temperature_unit: temp_unit,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }
            })
            .collect();

        db::cooking::RecipeStep::save_all_for_recipe(pool, recipe.recipe_id, steps).await?;

        // Add equipment
        for equipment_input in &input.equipment {
            // Get or create equipment
            let equipment =
                match db::cooking::Equipment::get_by_name(pool, &equipment_input.equipment_name)
                    .await?
                {
                    Some(e) => e,
                    None => {
                        db::cooking::Equipment::create(
                            pool,
                            equipment_input.equipment_name.clone(),
                            None,
                            equipment_input.is_optional,
                        )
                        .await?
                    }
                };

            // Create recipe equipment
            db::cooking::RecipeEquipment::create(
                pool,
                recipe.recipe_id,
                equipment.equipment_id,
                equipment_input.is_optional,
                equipment_input.notes.clone(),
            )
            .await?;
        }

        // Add tags
        let mut tag_ids = Vec::new();
        for tag_name in &input.tags {
            // Get or create tag
            let tag = match db::cooking::Tag::get_by_name(pool, tag_name).await? {
                Some(t) => t,
                None => db::cooking::Tag::create(pool, tag_name.clone(), None).await?,
            };
            tag_ids.push(tag.tag_id);
        }

        if !tag_ids.is_empty() {
            db::cooking::RecipeTag::set_tags_for_recipe(pool, recipe.recipe_id, tag_ids).await?;
        }

        Ok(UpsertRecipeOutput {
            recipe_id: recipe.recipe_id.to_string(),
        })
    }
}
