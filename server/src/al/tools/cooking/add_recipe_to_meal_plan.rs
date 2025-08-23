use chrono::NaiveDate;
use db::cooking::{MealPlanEntry, MealType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    al::tools::{ThreadContext, Tool},
    AppState,
};

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
