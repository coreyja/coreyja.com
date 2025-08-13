use chrono::{DateTime, Utc};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Type};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "UPPERCASE")]
pub enum TemperatureUnit {
    #[serde(rename = "F")]
    F,
    #[serde(rename = "C")]
    C,
}

impl fmt::Display for TemperatureUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemperatureUnit::F => write!(f, "F"),
            TemperatureUnit::C => write!(f, "C"),
        }
    }
}

impl std::str::FromStr for TemperatureUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "F" => Ok(TemperatureUnit::F),
            "C" => Ok(TemperatureUnit::C),
            _ => Err(format!("Unknown temperature unit: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeStep {
    pub step_id: Uuid,
    pub recipe_id: Uuid,
    pub step_number: i32,
    pub instruction: String,
    pub duration: Option<i32>, // minutes
    pub temperature: Option<i32>,
    pub temperature_unit: Option<TemperatureUnit>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RecipeStep {
    pub async fn save_all_for_recipe(
        pool: &PgPool,
        recipe_id: Uuid,
        steps: Vec<(String, Option<i32>, Option<i32>, Option<TemperatureUnit>)>,
    ) -> Result<Vec<Self>> {
        let mut transaction = pool.begin().await?;

        sqlx::query!("DELETE FROM recipe_steps WHERE recipe_id = $1", recipe_id)
            .execute(&mut *transaction)
            .await?;

        let mut saved_steps = Vec::new();

        for (index, (instruction, duration, temperature, temperature_unit)) in
            steps.into_iter().enumerate()
        {
            let step = sqlx::query_as!(
                RecipeStep,
                r#"
                INSERT INTO recipe_steps (
                    recipe_id, step_number, instruction, duration, 
                    temperature, temperature_unit
                )
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING 
                    step_id,
                    recipe_id,
                    step_number,
                    instruction,
                    duration,
                    temperature,
                    temperature_unit as "temperature_unit: TemperatureUnit",
                    created_at,
                    updated_at
                "#,
                recipe_id,
                (index as i32) + 1,
                instruction,
                duration,
                temperature,
                temperature_unit as Option<TemperatureUnit>
            )
            .fetch_one(&mut *transaction)
            .await?;

            saved_steps.push(step);
        }

        transaction.commit().await?;

        Ok(saved_steps)
    }

    pub async fn get_by_recipe(pool: &PgPool, recipe_id: Uuid) -> Result<Vec<Self>> {
        let steps = sqlx::query_as!(
            RecipeStep,
            r#"
            SELECT 
                step_id,
                recipe_id,
                step_number,
                instruction,
                duration,
                temperature,
                temperature_unit as "temperature_unit: TemperatureUnit",
                created_at,
                updated_at
            FROM recipe_steps
            WHERE recipe_id = $1
            ORDER BY step_number
            "#,
            recipe_id
        )
        .fetch_all(pool)
        .await?;

        Ok(steps)
    }
}

// Note: step_ingredients uses recipe_ingredient_id, not ingredient_id directly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepIngredient {
    pub step_id: Uuid,
    pub recipe_ingredient_id: Uuid,
}

impl StepIngredient {
    pub async fn create(pool: &PgPool, step_id: Uuid, recipe_ingredient_id: Uuid) -> Result<Self> {
        let step_ingredient = sqlx::query_as!(
            StepIngredient,
            r#"
            INSERT INTO step_ingredients (step_id, recipe_ingredient_id)
            VALUES ($1, $2)
            RETURNING 
                step_id,
                recipe_ingredient_id
            "#,
            step_id,
            recipe_ingredient_id
        )
        .fetch_one(pool)
        .await?;

        Ok(step_ingredient)
    }

    pub async fn get_by_step(pool: &PgPool, step_id: Uuid) -> Result<Vec<Self>> {
        let ingredients = sqlx::query_as!(
            StepIngredient,
            r#"
            SELECT 
                step_id,
                recipe_ingredient_id
            FROM step_ingredients
            WHERE step_id = $1
            "#,
            step_id
        )
        .fetch_all(pool)
        .await?;

        Ok(ingredients)
    }

    pub async fn delete(pool: &PgPool, step_id: Uuid, recipe_ingredient_id: Uuid) -> Result<()> {
        sqlx::query!(
            "DELETE FROM step_ingredients WHERE step_id = $1 AND recipe_ingredient_id = $2",
            step_id,
            recipe_ingredient_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

// Note: step_equipment uses composite primary key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepEquipment {
    pub step_id: Uuid,
    pub equipment_id: Uuid,
}

impl StepEquipment {
    pub async fn create(pool: &PgPool, step_id: Uuid, equipment_id: Uuid) -> Result<Self> {
        let step_equipment = sqlx::query_as!(
            StepEquipment,
            r#"
            INSERT INTO step_equipment (step_id, equipment_id)
            VALUES ($1, $2)
            RETURNING 
                step_id,
                equipment_id
            "#,
            step_id,
            equipment_id
        )
        .fetch_one(pool)
        .await?;

        Ok(step_equipment)
    }

    pub async fn get_by_step(pool: &PgPool, step_id: Uuid) -> Result<Vec<Self>> {
        let equipment = sqlx::query_as!(
            StepEquipment,
            r#"
            SELECT 
                step_id,
                equipment_id
            FROM step_equipment
            WHERE step_id = $1
            "#,
            step_id
        )
        .fetch_all(pool)
        .await?;

        Ok(equipment)
    }

    pub async fn delete(pool: &PgPool, step_id: Uuid, equipment_id: Uuid) -> Result<()> {
        sqlx::query!(
            "DELETE FROM step_equipment WHERE step_id = $1 AND equipment_id = $2",
            step_id,
            equipment_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
