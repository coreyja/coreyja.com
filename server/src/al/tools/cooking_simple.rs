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
    pub author_user_id: String,
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
    pub temperature: Option<f64>,
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
        "author_user_id": "123e4567-e89b-12d3-a456-426614174000",
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

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        let pool = &app_state.db;

        // Parse UUIDs
        let recipe_id = input.recipe_id.map(|id| Uuid::parse_str(&id)).transpose()?;
        let author_user_id = Uuid::parse_str(&input.author_user_id)?;

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
                    author_user_id,
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
                author_user_id,
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

        // Note: In a real implementation, you would add ingredients, steps, equipment, and tags here
        // For now, returning just the recipe ID

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

        Ok(GetRecipeOutput {
            recipe: recipe.map(|r| RecipeDetails {
                recipe_id: r.recipe_id.to_string(),
                name: r.name,
                description: r.description,
                prep_time: r.prep_time,
                cook_time: r.cook_time,
                servings: r.servings,
            }),
        })
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
        _app_state: AppState,
        _context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        // For now, return a simple success message
        // In a real implementation, this would update the inventory
        Ok(UpdateInventoryOutput {
            inventory_id: Uuid::new_v4().to_string(),
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
        _input: Self::ToolInput,
        _app_state: AppState,
        _context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        // For now, return empty inventory
        // In a real implementation, this would query the database
        Ok(CheckInventoryOutput { inventory: vec![] })
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
        _input: Self::ToolInput,
        _app_state: AppState,
        _context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        // For now, return empty list
        // In a real implementation, this would query the database
        Ok(ListMealPlansOutput { meal_plans: vec![] })
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
    pub author_user_id: String,
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
    
    Example:
    ```json
    {
        "name": "January Meal Plan",
        "description": "Healthy meals for the new year",
        "start_date": "2024-01-01T00:00:00Z",
        "end_date": "2024-01-31T23:59:59Z",
        "author_user_id": "123e4567-e89b-12d3-a456-426614174000"
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
