use db::cooking::Recipe;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    al::tools::{ThreadContext, Tool},
    AppState,
};

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
