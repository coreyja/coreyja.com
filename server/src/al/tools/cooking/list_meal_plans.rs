use chrono::NaiveDate;
use db::cooking::{MealPlan, MealType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    al::tools::{ThreadContext, Tool},
    AppState,
};

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
