use chrono::NaiveDate;
use db::cooking::MealPlan;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    al::tools::{ThreadContext, Tool},
    AppState,
};

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
