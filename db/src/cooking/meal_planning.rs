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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealPlan {
    pub meal_plan_id: Uuid,
    pub user_id: Uuid,
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
        user_id: Uuid,
        name: String,
        start_date: NaiveDate,
        end_date: NaiveDate,
        notes: Option<String>,
    ) -> Result<Self> {
        let meal_plan = sqlx::query_as!(
            MealPlan,
            r#"
            INSERT INTO meal_plans (
                user_id, name, start_date, end_date, notes
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING 
                meal_plan_id,
                user_id,
                name,
                start_date,
                end_date,
                notes,
                created_at,
                updated_at
            "#,
            user_id,
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
                user_id,
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
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<Self>> {
        let meal_plans = sqlx::query_as!(
            MealPlan,
            r#"
            SELECT 
                meal_plan_id,
                user_id,
                name,
                start_date,
                end_date,
                notes,
                created_at,
                updated_at
            FROM meal_plans
            WHERE user_id = $1
                AND start_date <= $3
                AND end_date >= $2
            ORDER BY start_date
            "#,
            user_id,
            start_date,
            end_date
        )
        .fetch_all(pool)
        .await?;

        Ok(meal_plans)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealPlanEntry {
    pub meal_plan_entry_id: Uuid,
    pub meal_plan_id: Uuid,
    pub recipe_id: Uuid,
    pub planned_date: NaiveDate,
    pub meal_type: MealType,
    pub servings: i32,
    pub notes: Option<String>,
    pub is_prepared: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MealPlanEntry {
    pub async fn create(
        pool: &PgPool,
        meal_plan_id: Uuid,
        recipe_id: Uuid,
        planned_date: NaiveDate,
        meal_type: MealType,
        servings: i32,
        notes: Option<String>,
    ) -> Result<Self> {
        let entry = sqlx::query_as!(
            MealPlanEntry,
            r#"
            INSERT INTO meal_plan_entries (
                meal_plan_id, recipe_id, planned_date, meal_type, 
                servings, notes, is_prepared
            )
            VALUES ($1, $2, $3, $4, $5, $6, false)
            RETURNING 
                meal_plan_entry_id,
                meal_plan_id,
                recipe_id,
                planned_date,
                meal_type as "meal_type: MealType",
                servings,
                notes,
                is_prepared,
                created_at,
                updated_at
            "#,
            meal_plan_id,
            recipe_id,
            planned_date,
            meal_type as MealType,
            servings,
            notes
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
                meal_plan_entry_id,
                meal_plan_id,
                recipe_id,
                planned_date,
                meal_type as "meal_type: MealType",
                servings,
                notes,
                is_prepared,
                created_at,
                updated_at
            FROM meal_plan_entries
            WHERE meal_plan_id = $1
            ORDER BY planned_date, meal_type
            "#,
            meal_plan_id
        )
        .fetch_all(pool)
        .await?;

        Ok(entries)
    }

    pub async fn get_by_date(
        pool: &PgPool,
        user_id: Uuid,
        planned_date: NaiveDate,
    ) -> Result<Vec<Self>> {
        let entries = sqlx::query_as!(
            MealPlanEntry,
            r#"
            SELECT 
                mpe.meal_plan_entry_id,
                mpe.meal_plan_id,
                mpe.recipe_id,
                mpe.planned_date,
                mpe.meal_type as "meal_type: MealType",
                mpe.servings,
                mpe.notes,
                mpe.is_prepared,
                mpe.created_at,
                mpe.updated_at
            FROM meal_plan_entries mpe
            JOIN meal_plans mp ON mpe.meal_plan_id = mp.meal_plan_id
            WHERE mp.user_id = $1
                AND mpe.planned_date = $2
            ORDER BY mpe.meal_type
            "#,
            user_id,
            planned_date
        )
        .fetch_all(pool)
        .await?;

        Ok(entries)
    }

    pub async fn mark_prepared(&self, pool: &PgPool) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE meal_plan_entries
            SET is_prepared = true,
                updated_at = NOW()
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
        planned_date: NaiveDate,
        meal_type: MealType,
        servings: i32,
        notes: Option<String>,
    ) -> Result<MealPlanEntry> {
        MealPlanEntry::create(
            pool,
            self.meal_plan_id,
            recipe_id,
            planned_date,
            meal_type,
            servings,
            notes,
        )
        .await
    }
}
