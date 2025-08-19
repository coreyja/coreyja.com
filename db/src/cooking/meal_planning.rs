use chrono::{DateTime, NaiveDate, Utc};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Type};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "snake_case")]
pub enum MealType {
    #[serde(rename = "breakfast")]
    Breakfast,
    #[serde(rename = "lunch")]
    Lunch,
    #[serde(rename = "dinner")]
    Dinner,
    #[serde(rename = "snack")]
    Snack,
}

impl fmt::Display for MealType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MealType::Breakfast => write!(f, "breakfast"),
            MealType::Lunch => write!(f, "lunch"),
            MealType::Dinner => write!(f, "dinner"),
            MealType::Snack => write!(f, "snack"),
        }
    }
}

impl std::str::FromStr for MealType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "breakfast" => Ok(MealType::Breakfast),
            "lunch" => Ok(MealType::Lunch),
            "dinner" => Ok(MealType::Dinner),
            "snack" => Ok(MealType::Snack),
            _ => Err(format!("Unknown meal type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MealPlan {
    pub meal_plan_id: Uuid,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MealPlan {
    pub async fn create(
        pool: &PgPool,
        name: String,
        start_date: NaiveDate,
        end_date: NaiveDate,
        notes: Option<String>,
    ) -> Result<Self> {
        let meal_plan = sqlx::query_as!(
            MealPlan,
            r#"
            INSERT INTO meal_plans (
                name, start_date, end_date, notes
            )
            VALUES ($1, $2, $3, $4)
            RETURNING 
                meal_plan_id,
                name,
                start_date,
                end_date,
                notes,
                created_at,
                updated_at
            "#,
            name,
            start_date,
            end_date,
            notes
        )
        .fetch_one(pool)
        .await?;

        Ok(meal_plan)
    }

    pub async fn get_by_id(pool: &PgPool, meal_plan_id: Uuid) -> Result<Option<Self>> {
        let meal_plan = sqlx::query_as!(
            MealPlan,
            r#"
            SELECT 
                meal_plan_id,
                name,
                start_date,
                end_date,
                notes,
                created_at,
                updated_at
            FROM meal_plans
            WHERE meal_plan_id = $1
            "#,
            meal_plan_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(meal_plan)
    }

    pub async fn get_by_date_range(
        pool: &PgPool,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<Self>> {
        let meal_plans = sqlx::query_as!(
            MealPlan,
            r#"
            SELECT 
                meal_plan_id,
                name,
                start_date,
                end_date,
                notes,
                created_at,
                updated_at
            FROM meal_plans
            WHERE start_date <= $2
                AND end_date >= $1
            ORDER BY start_date
            "#,
            start_date,
            end_date
        )
        .fetch_all(pool)
        .await?;

        Ok(meal_plans)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MealPlanEntry {
    pub meal_plan_entry_id: Uuid,
    pub meal_plan_id: Uuid,
    pub recipe_id: Uuid,
    pub date: NaiveDate,
    pub meal_type: Option<MealType>,
    pub servings_override: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MealPlanEntry {
    pub async fn create(
        pool: &PgPool,
        meal_plan_id: Uuid,
        recipe_id: Uuid,
        date: NaiveDate,
        meal_type: Option<MealType>,
        servings_override: Option<i32>,
    ) -> Result<Self> {
        let meal_type_str = meal_type.as_ref().map(std::string::ToString::to_string);

        let entry = sqlx::query_as!(
            MealPlanEntry,
            r#"
            INSERT INTO meal_plan_entries (
                meal_plan_id, recipe_id, date, meal_type, 
                servings_override
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING 
                meal_plan_entry_id as "meal_plan_entry_id!",
                meal_plan_id as "meal_plan_id!",
                recipe_id as "recipe_id!",
                date,
                meal_type as "meal_type: MealType",
                servings_override,
                created_at,
                updated_at
            "#,
            meal_plan_id,
            recipe_id,
            date,
            meal_type_str,
            servings_override
        )
        .fetch_one(pool)
        .await?;

        Ok(entry)
    }

    pub async fn get_by_meal_plan(pool: &PgPool, meal_plan_id: Uuid) -> Result<Vec<Self>> {
        let entries = sqlx::query_as!(
            MealPlanEntry,
            r#"
            SELECT 
                meal_plan_entry_id as "meal_plan_entry_id!",
                meal_plan_id as "meal_plan_id!",
                recipe_id as "recipe_id!",
                date,
                meal_type as "meal_type: MealType",
                servings_override,
                created_at,
                updated_at
            FROM meal_plan_entries
            WHERE meal_plan_id = $1
            ORDER BY date, meal_type
            "#,
            meal_plan_id
        )
        .fetch_all(pool)
        .await?;

        Ok(entries)
    }

    pub async fn get_by_date(pool: &PgPool, date: NaiveDate) -> Result<Vec<Self>> {
        let entries = sqlx::query_as!(
            MealPlanEntry,
            r#"
            SELECT 
                meal_plan_entry_id as "meal_plan_entry_id!",
                meal_plan_id as "meal_plan_id!",
                recipe_id as "recipe_id!",
                date,
                meal_type as "meal_type: MealType",
                servings_override,
                created_at,
                updated_at
            FROM meal_plan_entries
            WHERE date = $1
            ORDER BY meal_type
            "#,
            date
        )
        .fetch_all(pool)
        .await?;

        Ok(entries)
    }

    pub async fn delete(&self, pool: &PgPool) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM meal_plan_entries
            WHERE meal_plan_entry_id = $1
            "#,
            self.meal_plan_entry_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

impl MealPlan {
    pub async fn add_entry(
        &self,
        pool: &PgPool,
        recipe_id: Uuid,
        date: NaiveDate,
        meal_type: Option<MealType>,
        servings_override: Option<i32>,
    ) -> Result<MealPlanEntry> {
        MealPlanEntry::create(
            pool,
            self.meal_plan_id,
            recipe_id,
            date,
            meal_type,
            servings_override,
        )
        .await
    }
}
