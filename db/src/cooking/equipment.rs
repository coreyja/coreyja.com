use chrono::{DateTime, Utc};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Type};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "snake_case")]
pub enum EquipmentCategory {
    #[serde(rename = "cookware")]
    Cookware,
    #[serde(rename = "bakeware")]
    Bakeware,
    #[serde(rename = "appliance")]
    Appliance,
    #[serde(rename = "tool")]
    Tool,
    #[serde(rename = "utensil")]
    Utensil,
    #[serde(rename = "measuring")]
    Measuring,
    #[serde(rename = "mixing")]
    Mixing,
    #[serde(rename = "cutting")]
    Cutting,
}

impl fmt::Display for EquipmentCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EquipmentCategory::Cookware => write!(f, "cookware"),
            EquipmentCategory::Bakeware => write!(f, "bakeware"),
            EquipmentCategory::Appliance => write!(f, "appliance"),
            EquipmentCategory::Tool => write!(f, "tool"),
            EquipmentCategory::Utensil => write!(f, "utensil"),
            EquipmentCategory::Measuring => write!(f, "measuring"),
            EquipmentCategory::Mixing => write!(f, "mixing"),
            EquipmentCategory::Cutting => write!(f, "cutting"),
        }
    }
}

impl std::str::FromStr for EquipmentCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cookware" => Ok(EquipmentCategory::Cookware),
            "bakeware" => Ok(EquipmentCategory::Bakeware),
            "appliance" => Ok(EquipmentCategory::Appliance),
            "tool" => Ok(EquipmentCategory::Tool),
            "utensil" => Ok(EquipmentCategory::Utensil),
            "measuring" => Ok(EquipmentCategory::Measuring),
            "mixing" => Ok(EquipmentCategory::Mixing),
            "cutting" => Ok(EquipmentCategory::Cutting),
            _ => Err(format!("Unknown equipment category: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Equipment {
    pub equipment_id: Uuid,
    pub name: String,
    pub category: Option<EquipmentCategory>,
    pub is_optional: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Equipment {
    pub async fn create(
        pool: &PgPool,
        name: String,
        category: Option<EquipmentCategory>,
        is_optional: bool,
    ) -> Result<Self> {
        let equipment = sqlx::query_as!(
            Equipment,
            r#"
            INSERT INTO equipment (name, category, is_optional)
            VALUES ($1, $2, $3)
            RETURNING 
                equipment_id,
                name,
                category as "category: EquipmentCategory",
                is_optional,
                created_at,
                updated_at
            "#,
            name,
            category as Option<EquipmentCategory>,
            is_optional
        )
        .fetch_one(pool)
        .await?;

        Ok(equipment)
    }

    pub async fn get_by_id(pool: &PgPool, equipment_id: Uuid) -> Result<Option<Self>> {
        let equipment = sqlx::query_as!(
            Equipment,
            r#"
            SELECT 
                equipment_id,
                name,
                category as "category: EquipmentCategory",
                is_optional,
                created_at,
                updated_at
            FROM equipment
            WHERE equipment_id = $1
            "#,
            equipment_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(equipment)
    }

    pub async fn get_by_name(pool: &PgPool, name: &str) -> Result<Option<Self>> {
        let equipment = sqlx::query_as!(
            Equipment,
            r#"
            SELECT 
                equipment_id,
                name,
                category as "category: EquipmentCategory",
                is_optional,
                created_at,
                updated_at
            FROM equipment
            WHERE name = $1
            "#,
            name
        )
        .fetch_optional(pool)
        .await?;

        Ok(equipment)
    }
}

// Note: recipe_equipment uses composite primary key (recipe_id, equipment_id)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeEquipment {
    pub recipe_id: Uuid,
    pub equipment_id: Uuid,
    pub is_optional: bool,
    pub notes: Option<String>,
}

impl RecipeEquipment {
    pub async fn create(
        pool: &PgPool,
        recipe_id: Uuid,
        equipment_id: Uuid,
        is_optional: bool,
        notes: Option<String>,
    ) -> Result<Self> {
        let recipe_equipment = sqlx::query_as!(
            RecipeEquipment,
            r#"
            INSERT INTO recipe_equipment (
                recipe_id, equipment_id, is_optional, notes
            )
            VALUES ($1, $2, $3, $4)
            RETURNING 
                recipe_id,
                equipment_id,
                is_optional,
                notes
            "#,
            recipe_id,
            equipment_id,
            is_optional,
            notes
        )
        .fetch_one(pool)
        .await?;

        Ok(recipe_equipment)
    }

    pub async fn get_by_recipe(pool: &PgPool, recipe_id: Uuid) -> Result<Vec<Self>> {
        let equipment = sqlx::query_as!(
            RecipeEquipment,
            r#"
            SELECT 
                recipe_id,
                equipment_id,
                is_optional,
                notes
            FROM recipe_equipment
            WHERE recipe_id = $1
            "#,
            recipe_id
        )
        .fetch_all(pool)
        .await?;

        Ok(equipment)
    }

    pub async fn delete(pool: &PgPool, recipe_id: Uuid, equipment_id: Uuid) -> Result<()> {
        sqlx::query!(
            "DELETE FROM recipe_equipment WHERE recipe_id = $1 AND equipment_id = $2",
            recipe_id,
            equipment_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
