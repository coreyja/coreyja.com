use std::str::FromStr;

use chrono::NaiveDate;
use db::cooking::{MealPlan, MealPlanEntry, MealType, Recipe};
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

// GetRecipe Tool
#[derive(Clone, Debug)]
pub struct GetRecipe;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetRecipeInput {
    pub recipe_id: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RecipeDetails {
    pub recipe_id: String,
    pub name: String,
    pub description: Option<String>,
    pub prep_time: Option<i32>,
    pub cook_time: Option<i32>,
    pub servings: i32,
    pub ingredients: Vec<RecipeIngredientDetails>,
    pub steps: Vec<RecipeStepDetails>,
    pub equipment: Vec<RecipeEquipmentDetails>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RecipeIngredientDetails {
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
pub struct RecipeStepDetails {
    pub step_number: i32,
    pub instruction: String,
    pub duration: Option<i32>,
    pub temperature: Option<f64>,
    pub temperature_unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RecipeEquipmentDetails {
    pub equipment_name: String,
    pub is_optional: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetRecipeOutput {
    pub recipe: Option<RecipeDetails>,
}

#[async_trait::async_trait]
impl Tool for GetRecipe {
    const NAME: &'static str = "get_recipe";
    const DESCRIPTION: &'static str = r#"
    Retrieve a recipe by ID or name with all details including ingredients, steps, equipment, and tags.

    Provide either recipe_id or name (one is required).

    Example by ID:
    ```json
    {
        "recipe_id": "123e4567-e89b-12d3-a456-426614174000"
    }
    ```

    Example by name:
    ```json
    {
        "name": "Chocolate Chip Cookies"
    }
    ```
    "#;

    type ToolInput = GetRecipeInput;
    type ToolOutput = GetRecipeOutput;

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        _context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        let pool = &app_state.db;

        let recipe_id = if let Some(id_str) = input.recipe_id {
            Some(Uuid::parse_str(&id_str)?)
        } else if let Some(name) = input.name {
            let result = sqlx::query!(
                r#"SELECT recipe_id FROM recipes WHERE LOWER(name) = LOWER($1) LIMIT 1"#,
                name
            )
            .fetch_optional(pool)
            .await?;
            result.map(|r| r.recipe_id)
        } else {
            return Err(cja::color_eyre::eyre::eyre!(
                "Either recipe_id or name must be provided"
            ));
        };

        let recipe = if let Some(id) = recipe_id {
            Recipe::get_by_id(pool, id).await?
        } else {
            None
        };

        if let Some(recipe) = recipe {
            // Get ingredients
            let ingredients =
                db::cooking::RecipeIngredient::get_by_recipe(pool, recipe.recipe_id).await?;
            let mut ingredient_details = Vec::new();

            for ri in ingredients {
                // Get ingredient and unit details
                let ingredient = db::cooking::Ingredient::get_by_id(pool, ri.ingredient_id)
                    .await?
                    .ok_or_else(|| cja::color_eyre::eyre::eyre!("Ingredient not found"))?;
                let unit = db::cooking::Unit::get_by_id(pool, ri.unit_id)
                    .await?
                    .ok_or_else(|| cja::color_eyre::eyre::eyre!("Unit not found"))?;

                ingredient_details.push(RecipeIngredientDetails {
                    ingredient_name: ingredient.name,
                    quantity: ri.quantity.to_string().parse::<f64>()?,
                    unit_name: unit.name,
                    is_optional: ri.is_optional,
                    notes: ri.notes,
                    ingredient_group: ri.ingredient_group,
                    preparation: ri.preparation.map(|p| p.to_string()),
                    temperature: ri.temperature.map(|t| t.to_string()),
                });
            }

            // Get steps
            let steps = db::cooking::RecipeStep::get_by_recipe(pool, recipe.recipe_id).await?;
            let step_details = steps
                .into_iter()
                .map(|step| RecipeStepDetails {
                    step_number: step.step_number,
                    instruction: step.instruction,
                    duration: step.duration,
                    temperature: step
                        .temperature
                        .and_then(|t| t.to_string().parse::<f64>().ok()),
                    temperature_unit: step.temperature_unit.map(|t| t.to_string()),
                })
                .collect();

            // Get equipment
            let equipment =
                db::cooking::RecipeEquipment::get_by_recipe(pool, recipe.recipe_id).await?;
            let mut equipment_details = Vec::new();

            for re in equipment {
                let equip = db::cooking::Equipment::get_by_id(pool, re.equipment_id)
                    .await?
                    .ok_or_else(|| cja::color_eyre::eyre::eyre!("Equipment not found"))?;

                equipment_details.push(RecipeEquipmentDetails {
                    equipment_name: equip.name,
                    is_optional: re.is_optional,
                    notes: re.notes,
                });
            }

            // Get tags
            let tags = db::cooking::RecipeTag::get_by_recipe(pool, recipe.recipe_id).await?;
            let mut tag_names = Vec::new();

            for rt in tags {
                let tag = db::cooking::Tag::get_by_id(pool, rt.tag_id)
                    .await?
                    .ok_or_else(|| cja::color_eyre::eyre::eyre!("Tag not found"))?;
                tag_names.push(tag.name);
            }

            Ok(GetRecipeOutput {
                recipe: Some(RecipeDetails {
                    recipe_id: recipe.recipe_id.to_string(),
                    name: recipe.name,
                    description: recipe.description,
                    prep_time: recipe.prep_time,
                    cook_time: recipe.cook_time,
                    servings: recipe.servings,
                    ingredients: ingredient_details,
                    steps: step_details,
                    equipment: equipment_details,
                    tags: tag_names,
                }),
            })
        } else {
            Ok(GetRecipeOutput { recipe: None })
        }
    }
}

// UpdateInventory Tool
#[derive(Clone, Debug)]
pub struct UpdateInventory;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateInventoryInput {
    pub ingredient_name: String,
    pub quantity: f64,
    pub unit_name: String,
    pub confidence_level: Option<String>,
    pub location_name: Option<String>,
    pub expiration_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateInventoryOutput {
    pub inventory_id: String,
    pub message: String,
}

#[async_trait::async_trait]
impl Tool for UpdateInventory {
    const NAME: &'static str = "update_inventory";
    const DESCRIPTION: &'static str = r#"
    Add or update ingredient quantities in inventory.

    Will create ingredient, unit, or location if they don't exist.
    Confidence levels: exact, high, medium, low, empty

    Example:
    ```json
    {
        "ingredient_name": "all-purpose flour",
        "quantity": 5,
        "unit_name": "pounds",
        "confidence_level": "high",
        "location_name": "Pantry Shelf 2",
        "expiration_date": "2024-12-31T00:00:00Z"
    }
    ```
    "#;

    type ToolInput = UpdateInventoryInput;
    type ToolOutput = UpdateInventoryOutput;

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        _context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        let pool = &app_state.db;

        // Get or create ingredient
        let ingredient =
            match db::cooking::Ingredient::get_by_name(pool, &input.ingredient_name).await? {
                Some(i) => i,
                None => {
                    db::cooking::Ingredient::create(pool, input.ingredient_name.clone(), None, None)
                        .await?
                }
            };

        // Get or create unit
        let unit = match db::cooking::Unit::get_by_name(pool, &input.unit_name).await? {
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
                    input.unit_name,
                    Some("volume")
                )
                .fetch_one(pool)
                .await?
            }
        };

        // Get or create location
        let location_id = if let Some(location_name) = input.location_name {
            // Try to find existing location by name
            let existing = sqlx::query!(
                "SELECT location_id FROM locations WHERE name = $1 LIMIT 1",
                location_name
            )
            .fetch_optional(pool)
            .await?;

            if let Some(loc) = existing {
                Some(loc.location_id)
            } else {
                // Create new location with pantry as default type
                let location = db::cooking::Location::create(
                    pool,
                    location_name,
                    None,
                    Some(db::cooking::LocationType::Pantry),
                )
                .await?;
                Some(location.location_id)
            }
        } else {
            None
        };

        // Parse confidence level
        let confidence = input
            .confidence_level
            .as_ref()
            .and_then(|c| db::cooking::ConfidenceLevel::from_str(c).ok())
            .unwrap_or(db::cooking::ConfidenceLevel::Medium);

        // Parse expiration date
        let expiration_date = input.expiration_date.as_ref().and_then(|d| {
            chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .or_else(|_| {
                    chrono::NaiveDateTime::parse_from_str(d, "%Y-%m-%dT%H:%M:%SZ")
                        .map(|dt| dt.date())
                })
                .ok()
        });

        // Convert quantity to BigDecimal
        let quantity = sqlx::types::BigDecimal::from_str(&input.quantity.to_string())?;

        // Check if inventory exists for this ingredient
        let existing_inventory =
            db::cooking::Inventory::get_by_ingredient(pool, ingredient.ingredient_id).await?;

        let inventory = if let Some(inv) = existing_inventory.into_iter().next() {
            // Update existing inventory
            inv.update(
                pool,
                Some(quantity),
                Some(unit.unit_id),
                Some(confidence),
                expiration_date,
                location_id,
                None, // notes
            )
            .await?
        } else {
            // Create new inventory
            db::cooking::Inventory::create(
                pool,
                ingredient.ingredient_id,
                quantity,
                Some(unit.unit_id),
                Some(confidence),
                expiration_date,
                location_id,
                None, // notes
            )
            .await?
        };

        Ok(UpdateInventoryOutput {
            inventory_id: inventory.inventory_id.to_string(),
            message: format!(
                "Updated inventory for {} with {} {}",
                input.ingredient_name, input.quantity, input.unit_name
            ),
        })
    }
}

// CheckInventory Tool
#[derive(Clone, Debug)]
pub struct CheckInventory;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CheckInventoryInput {
    pub ingredient_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InventoryItem {
    pub ingredient_name: String,
    pub quantity: f64,
    pub unit_name: String,
    pub confidence_level: String,
    pub location_name: Option<String>,
    pub expiration_date: Option<String>,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CheckInventoryOutput {
    pub inventory: Vec<InventoryItem>,
}

#[async_trait::async_trait]
impl Tool for CheckInventory {
    const NAME: &'static str = "check_inventory";
    const DESCRIPTION: &'static str = r#"
    Check what ingredients are available and their quantities.

    If ingredient_names is empty, returns all inventory items.

    Example for specific items:
    ```json
    {
        "ingredient_names": ["flour", "sugar", "eggs"]
    }
    ```

    Example for all items:
    ```json
    {
        "ingredient_names": []
    }
    ```
    "#;

    type ToolInput = CheckInventoryInput;
    type ToolOutput = CheckInventoryOutput;

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        _context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        let pool = &app_state.db;

        // Build ingredient filter
        let ingredient_filter = if input.ingredient_names.is_empty() {
            None
        } else {
            Some(input.ingredient_names.clone())
        };

        // Get inventory items with optional filter
        let inventory_items = sqlx::query!(
            r#"
            SELECT
                i.inventory_id,
                ing.name as ingredient_name,
                i.quantity,
                i.unit_id,
                i.confidence_level,
                i.location_id,
                i.expiration_date,
                i.updated_at
            FROM inventory i
            JOIN ingredients ing ON i.ingredient_id = ing.ingredient_id
            WHERE ($1::text[] IS NULL OR LOWER(ing.name) = ANY(
                SELECT LOWER(unnest($1::text[]))
            ))
            ORDER BY ing.name
            "#,
            ingredient_filter.as_deref()
        )
        .fetch_all(pool)
        .await?;

        let mut inventory = Vec::new();

        for item in inventory_items {
            // Get unit name if unit_id exists
            let unit_name = if let Some(unit_id) = item.unit_id {
                sqlx::query!("SELECT name FROM units WHERE unit_id = $1", unit_id)
                    .fetch_optional(pool)
                    .await?
                    .map_or_else(|| "units".to_string(), |u| u.name)
            } else {
                "units".to_string()
            };

            // Get location name if location_id exists
            let location_name = if let Some(location_id) = item.location_id {
                sqlx::query!(
                    "SELECT name FROM locations WHERE location_id = $1",
                    location_id
                )
                .fetch_optional(pool)
                .await?
                .map(|l| l.name)
            } else {
                None
            };

            inventory.push(InventoryItem {
                ingredient_name: item.ingredient_name,
                quantity: item.quantity.to_string().parse::<f64>()?,
                unit_name,
                confidence_level: item.confidence_level.unwrap_or("medium".to_string()),
                location_name,
                expiration_date: item.expiration_date.map(|d| d.to_string()),
                last_updated: item.updated_at.to_string(),
            });
        }

        Ok(CheckInventoryOutput { inventory })
    }
}

// ListMealPlans Tool
#[derive(Clone, Debug)]
pub struct ListMealPlans;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListMealPlansInput {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub include_past: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MealPlanItem {
    pub meal_plan_id: String,
    pub name: String,
    pub description: Option<String>,
    pub start_date: String,
    pub end_date: String,
    pub entries: Vec<MealPlanEntryItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MealPlanEntryItem {
    pub date: String,
    pub meal_type: String,
    pub recipe_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListMealPlansOutput {
    pub meal_plans: Vec<MealPlanItem>,
}

#[async_trait::async_trait]
impl Tool for ListMealPlans {
    const NAME: &'static str = "list_meal_plans";
    const DESCRIPTION: &'static str = r#"
    List current and upcoming meal plans with their scheduled recipes.

    - start_date: defaults to today
    - end_date: defaults to 7 days from start_date
    - include_past: defaults to false

    Example:
    ```json
    {
        "start_date": "2024-01-01T00:00:00Z",
        "end_date": "2024-01-31T00:00:00Z",
        "include_past": false
    }
    ```
    "#;

    type ToolInput = ListMealPlansInput;
    type ToolOutput = ListMealPlansOutput;

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        _context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        let pool = &app_state.db;

        // Parse dates or use defaults
        let start_date = if let Some(date_str) = input.start_date {
            NaiveDate::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%SZ")
                .or_else(|_| NaiveDate::parse_from_str(&date_str, "%Y-%m-%d"))?
        } else {
            chrono::Local::now().date_naive()
        };

        let end_date = if let Some(date_str) = input.end_date {
            NaiveDate::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%SZ")
                .or_else(|_| NaiveDate::parse_from_str(&date_str, "%Y-%m-%d"))?
        } else {
            start_date + chrono::Duration::days(7)
        };

        // Get meal plans
        let meal_plans = MealPlan::get_by_date_range(pool, start_date, end_date).await?;

        let mut result = Vec::new();

        for meal_plan in meal_plans {
            // Get entries for this meal plan
            let entries = sqlx::query!(
                r#"
                SELECT
                    mpe.date,
                    mpe.meal_type,
                    r.name as recipe_name
                FROM meal_plan_entries mpe
                JOIN recipes r ON mpe.recipe_id = r.recipe_id
                WHERE mpe.meal_plan_id = $1
                ORDER BY mpe.date, mpe.meal_type
                "#,
                meal_plan.meal_plan_id
            )
            .fetch_all(pool)
            .await?;

            let entry_details = entries
                .into_iter()
                .map(|entry| MealPlanEntryItem {
                    date: entry.date.to_string(),
                    meal_type: entry.meal_type.unwrap_or(MealType::Dinner.to_string()),
                    recipe_name: entry.recipe_name,
                })
                .collect();

            result.push(MealPlanItem {
                meal_plan_id: meal_plan.meal_plan_id.to_string(),
                name: meal_plan.name,
                description: meal_plan.notes,
                start_date: meal_plan.start_date.to_string(),
                end_date: meal_plan.end_date.to_string(),
                entries: entry_details,
            });
        }

        Ok(ListMealPlansOutput { meal_plans: result })
    }
}

// CreateMealPlan Tool
#[derive(Clone, Debug)]
pub struct CreateMealPlan;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateMealPlanInput {
    pub name: String,
    pub description: Option<String>,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateMealPlanOutput {
    pub meal_plan_id: String,
}

#[async_trait::async_trait]
impl Tool for CreateMealPlan {
    const NAME: &'static str = "create_meal_plan";
    const DESCRIPTION: &'static str = r#"
    Create a meal plan for a date range.
    We are parsing the date strings with "%Y-%m-%d"
    Follow the example format below:

    Example:
    ```json
    {
        "name": "January Meal Plan",
        "description": "Healthy meals for the new year",
        "start_date": "2024-01-01",
        "end_date": "2024-01-31"
    }
    ```
    "#;

    type ToolInput = CreateMealPlanInput;
    type ToolOutput = CreateMealPlanOutput;

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        _context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        let pool = &app_state.db;

        // Convert DateTime to NaiveDate
        let start_date = NaiveDate::parse_from_str(&input.start_date, "%Y-%m-%d")?;
        let end_date = NaiveDate::parse_from_str(&input.end_date, "%Y-%m-%d")?;

        let meal_plan =
            MealPlan::create(pool, input.name, start_date, end_date, input.description).await?;

        Ok(CreateMealPlanOutput {
            meal_plan_id: meal_plan.meal_plan_id.to_string(),
        })
    }
}

// AddRecipeToMealPlan Tool
#[derive(Clone, Debug)]
pub struct AddRecipeToMealPlan;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddRecipeToMealPlanInput {
    pub meal_plan_id: String,
    pub recipe_id: Option<String>,
    pub recipe_name: Option<String>,
    pub date: String,
    pub meal_type: String,
    pub servings: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddRecipeToMealPlanOutput {
    pub entry_id: String,
}

#[async_trait::async_trait]
impl Tool for AddRecipeToMealPlan {
    const NAME: &'static str = "add_recipe_to_meal_plan";
    const DESCRIPTION: &'static str = r#"
    Schedule recipes for specific meals in a meal plan.

    Provide either recipe_id or recipe_name.
    Meal types: breakfast, lunch, dinner, snack

    Example:
    ```json
    {
        "meal_plan_id": "123e4567-e89b-12d3-a456-426614174000",
        "recipe_name": "Chicken Stir Fry",
        "date": "2024-01-15",
        "meal_type": "dinner",
        "servings": 4,
        "notes": "Double the vegetables"
    }
    ```
    "#;

    type ToolInput = AddRecipeToMealPlanInput;
    type ToolOutput = AddRecipeToMealPlanOutput;

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        _context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        let pool = &app_state.db;

        // Parse IDs and date
        let meal_plan_id = Uuid::parse_str(&input.meal_plan_id)?;
        let date = NaiveDate::parse_from_str(&input.date, "%Y-%m-%d")?;

        // Get recipe ID
        let recipe_id = if let Some(id_str) = input.recipe_id {
            Uuid::parse_str(&id_str)?
        } else if let Some(name) = input.recipe_name {
            let result = sqlx::query!(
                r#"SELECT recipe_id FROM recipes WHERE LOWER(name) = LOWER($1) LIMIT 1"#,
                name
            )
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| cja::color_eyre::eyre::eyre!("Recipe not found: {}", name))?;
            result.recipe_id
        } else {
            return Err(cja::color_eyre::eyre::eyre!(
                "Either recipe_id or recipe_name must be provided"
            ));
        };

        // Parse meal type
        let meal_type = match input.meal_type.to_lowercase().as_str() {
            "breakfast" => MealType::Breakfast,
            "lunch" => MealType::Lunch,
            "dinner" => MealType::Dinner,
            "snack" => MealType::Snack,
            _ => return Err(cja::color_eyre::eyre::eyre!("Invalid meal type")),
        };

        // Create meal plan entry
        let entry = MealPlanEntry::create(
            pool,
            meal_plan_id,
            recipe_id,
            date,
            Some(meal_type),
            input.servings,
        )
        .await?;

        Ok(AddRecipeToMealPlanOutput {
            entry_id: entry.meal_plan_entry_id.to_string(),
        })
    }
}
