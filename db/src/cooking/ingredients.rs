use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Type};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "snake_case")]
pub enum UnitType {
    #[serde(rename = "volume")]
    Volume,
    #[serde(rename = "weight")]
    Weight,
    #[serde(rename = "count")]
    Count,
}

impl fmt::Display for UnitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnitType::Volume => write!(f, "volume"),
            UnitType::Weight => write!(f, "weight"),
            UnitType::Count => write!(f, "count"),
        }
    }
}

impl std::str::FromStr for UnitType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "volume" => Ok(UnitType::Volume),
            "weight" => Ok(UnitType::Weight),
            "count" => Ok(UnitType::Count),
            _ => Err(format!("Unknown unit type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "snake_case")]
pub enum IngredientPreparation {
    #[serde(rename = "diced")]
    Diced,
    #[serde(rename = "minced")]
    Minced,
    #[serde(rename = "chopped")]
    Chopped,
    #[serde(rename = "sliced")]
    Sliced,
    #[serde(rename = "julienned")]
    Julienned,
    #[serde(rename = "grated")]
    Grated,
    #[serde(rename = "zested")]
    Zested,
    #[serde(rename = "crushed")]
    Crushed,
    #[serde(rename = "mashed")]
    Mashed,
    #[serde(rename = "whole")]
    Whole,
    #[serde(rename = "halved")]
    Halved,
    #[serde(rename = "quartered")]
    Quartered,
}

impl fmt::Display for IngredientPreparation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IngredientPreparation::Diced => write!(f, "diced"),
            IngredientPreparation::Minced => write!(f, "minced"),
            IngredientPreparation::Chopped => write!(f, "chopped"),
            IngredientPreparation::Sliced => write!(f, "sliced"),
            IngredientPreparation::Julienned => write!(f, "julienned"),
            IngredientPreparation::Grated => write!(f, "grated"),
            IngredientPreparation::Zested => write!(f, "zested"),
            IngredientPreparation::Crushed => write!(f, "crushed"),
            IngredientPreparation::Mashed => write!(f, "mashed"),
            IngredientPreparation::Whole => write!(f, "whole"),
            IngredientPreparation::Halved => write!(f, "halved"),
            IngredientPreparation::Quartered => write!(f, "quartered"),
        }
    }
}

impl std::str::FromStr for IngredientPreparation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "diced" => Ok(IngredientPreparation::Diced),
            "minced" => Ok(IngredientPreparation::Minced),
            "chopped" => Ok(IngredientPreparation::Chopped),
            "sliced" => Ok(IngredientPreparation::Sliced),
            "julienned" => Ok(IngredientPreparation::Julienned),
            "grated" => Ok(IngredientPreparation::Grated),
            "zested" => Ok(IngredientPreparation::Zested),
            "crushed" => Ok(IngredientPreparation::Crushed),
            "mashed" => Ok(IngredientPreparation::Mashed),
            "whole" => Ok(IngredientPreparation::Whole),
            "halved" => Ok(IngredientPreparation::Halved),
            "quartered" => Ok(IngredientPreparation::Quartered),
            _ => Err(format!("Unknown ingredient preparation: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "snake_case")]
pub enum IngredientTemperature {
    #[serde(rename = "room_temp")]
    RoomTemp,
    #[serde(rename = "chilled")]
    Chilled,
    #[serde(rename = "frozen")]
    Frozen,
    #[serde(rename = "melted")]
    Melted,
    #[serde(rename = "softened")]
    Softened,
}

impl fmt::Display for IngredientTemperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IngredientTemperature::RoomTemp => write!(f, "room_temp"),
            IngredientTemperature::Chilled => write!(f, "chilled"),
            IngredientTemperature::Frozen => write!(f, "frozen"),
            IngredientTemperature::Melted => write!(f, "melted"),
            IngredientTemperature::Softened => write!(f, "softened"),
        }
    }
}

impl std::str::FromStr for IngredientTemperature {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "room_temp" => Ok(IngredientTemperature::RoomTemp),
            "chilled" => Ok(IngredientTemperature::Chilled),
            "frozen" => Ok(IngredientTemperature::Frozen),
            "melted" => Ok(IngredientTemperature::Melted),
            "softened" => Ok(IngredientTemperature::Softened),
            _ => Err(format!("Unknown ingredient temperature: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Unit {
    pub unit_id: Uuid,
    pub name: String,
    #[sqlx(rename = "type")]
    pub unit_type: Option<UnitType>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Unit {
    pub async fn get_by_id(pool: &PgPool, unit_id: Uuid) -> Result<Option<Self>> {
        let unit = sqlx::query_as::<_, Unit>(
            "
            SELECT
                unit_id,
                name,
                type,
                created_at,
                updated_at
            FROM units
            WHERE unit_id = $1
            ",
        )
        .bind(unit_id)
        .fetch_optional(pool)
        .await?;

        Ok(unit)
    }

    pub async fn get_by_name(pool: &PgPool, name: &str) -> Result<Option<Self>> {
        let unit = sqlx::query_as::<_, Unit>(
            "
            SELECT
                unit_id,
                name,
                type,
                created_at,
                updated_at
            FROM units
            WHERE name = $1
            ",
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;

        Ok(unit)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ingredient {
    pub ingredient_id: Uuid,
    pub name: String,
    pub category: Option<String>,
    pub default_unit_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Ingredient {
    pub async fn create(
        pool: &PgPool,
        name: String,
        category: Option<String>,
        default_unit_id: Option<Uuid>,
    ) -> Result<Self> {
        let ingredient = sqlx::query_as!(
            Ingredient,
            r#"
            INSERT INTO ingredients (name, category, default_unit_id)
            VALUES ($1, $2, $3)
            RETURNING
                ingredient_id,
                name,
                category,
                default_unit_id,
                created_at,
                updated_at
            "#,
            name,
            category,
            default_unit_id
        )
        .fetch_one(pool)
        .await?;

        Ok(ingredient)
    }

    pub async fn get_by_id(pool: &PgPool, ingredient_id: Uuid) -> Result<Option<Self>> {
        let ingredient = sqlx::query_as!(
            Ingredient,
            r#"
            SELECT
                ingredient_id,
                name,
                category,
                default_unit_id,
                created_at,
                updated_at
            FROM ingredients
            WHERE ingredient_id = $1
            "#,
            ingredient_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(ingredient)
    }

    pub async fn get_by_name(pool: &PgPool, name: &str) -> Result<Option<Self>> {
        let ingredient = sqlx::query_as!(
            Ingredient,
            r#"
            SELECT
                ingredient_id,
                name,
                category,
                default_unit_id,
                created_at,
                updated_at
            FROM ingredients
            WHERE name = $1
            "#,
            name
        )
        .fetch_optional(pool)
        .await?;

        Ok(ingredient)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RecipeIngredient {
    pub recipe_ingredient_id: Uuid,
    pub recipe_id: Uuid,
    pub ingredient_id: Uuid,
    pub quantity: BigDecimal,
    pub unit_id: Uuid,
    pub display_order: Option<i32>,
    pub ingredient_group: Option<String>,
    pub preparation: Option<IngredientPreparation>,
    pub temperature: Option<IngredientTemperature>,
    pub is_optional: Option<bool>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RecipeIngredient {
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        pool: &PgPool,
        recipe_id: Uuid,
        ingredient_id: Uuid,
        quantity: BigDecimal,
        unit_id: Uuid,
        display_order: Option<i32>,
        ingredient_group: Option<String>,
        preparation: Option<IngredientPreparation>,
        temperature: Option<IngredientTemperature>,
        is_optional: Option<bool>,
        notes: Option<String>,
    ) -> Result<Self> {
        let preparation_str = preparation.as_ref().map(std::string::ToString::to_string);
        let temperature_str = temperature.as_ref().map(std::string::ToString::to_string);

        let recipe_ingredient = sqlx::query_as!(
            RecipeIngredient,
            r#"
            INSERT INTO recipe_ingredients (
                recipe_id, ingredient_id, quantity, unit_id, display_order,
                ingredient_group, preparation, temperature, is_optional, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING
                recipe_ingredient_id as "recipe_ingredient_id!",
                recipe_id as "recipe_id!",
                ingredient_id as "ingredient_id!",
                quantity,
                unit_id as "unit_id!",
                display_order,
                ingredient_group,
                preparation as "preparation: IngredientPreparation",
                temperature as "temperature: IngredientTemperature",
                is_optional,
                notes,
                created_at,
                updated_at
            "#,
            recipe_id,
            ingredient_id,
            quantity,
            unit_id,
            display_order,
            ingredient_group,
            preparation_str,
            temperature_str,
            is_optional,
            notes
        )
        .fetch_one(pool)
        .await?;

        Ok(recipe_ingredient)
    }

    pub async fn get_by_recipe(pool: &PgPool, recipe_id: Uuid) -> Result<Vec<Self>> {
        let ingredients = sqlx::query_as!(
            RecipeIngredient,
            r#"
            SELECT
                recipe_ingredient_id as "recipe_ingredient_id!",
                recipe_id as "recipe_id!",
                ingredient_id as "ingredient_id!",
                quantity,
                unit_id as "unit_id!",
                display_order,
                ingredient_group,
                preparation as "preparation: IngredientPreparation",
                temperature as "temperature: IngredientTemperature",
                is_optional,
                notes,
                created_at,
                updated_at
            FROM recipe_ingredients
            WHERE recipe_id = $1
            ORDER BY display_order
            "#,
            recipe_id
        )
        .fetch_all(pool)
        .await?;

        Ok(ingredients)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        pool: &PgPool,
        quantity: BigDecimal,
        unit_id: Uuid,
        display_order: Option<i32>,
        ingredient_group: Option<String>,
        preparation: Option<IngredientPreparation>,
        temperature: Option<IngredientTemperature>,
        is_optional: bool,
        notes: Option<String>,
    ) -> Result<Self> {
        let updated = sqlx::query_as!(
            RecipeIngredient,
            r#"
            UPDATE recipe_ingredients
            SET quantity = $2,
                unit_id = $3,
                display_order = $4,
                ingredient_group = $5,
                preparation = $6,
                temperature = $7,
                is_optional = $8,
                notes = $9,
                updated_at = NOW()
            WHERE recipe_ingredient_id = $1
            RETURNING
                recipe_ingredient_id as "recipe_ingredient_id!",
                recipe_id as "recipe_id!",
                ingredient_id as "ingredient_id!",
                quantity,
                unit_id as "unit_id!",
                display_order,
                ingredient_group,
                preparation as "preparation: IngredientPreparation",
                temperature as "temperature: IngredientTemperature",
                is_optional,
                notes,
                created_at,
                updated_at
            "#,
            self.recipe_ingredient_id,
            quantity,
            unit_id,
            display_order,
            ingredient_group,
            preparation as Option<IngredientPreparation>,
            temperature as Option<IngredientTemperature>,
            is_optional,
            notes
        )
        .fetch_one(pool)
        .await?;

        Ok(updated)
    }

    pub async fn delete(&self, pool: &PgPool) -> Result<()> {
        sqlx::query!(
            "DELETE FROM recipe_ingredients WHERE recipe_ingredient_id = $1",
            self.recipe_ingredient_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
