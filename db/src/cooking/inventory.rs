use chrono::{DateTime, NaiveDate, Utc};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Type};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "snake_case")]
pub enum LocationType {
    #[serde(rename = "fridge")]
    Fridge,
    #[serde(rename = "freezer")]
    Freezer,
    #[serde(rename = "pantry")]
    Pantry,
    #[serde(rename = "counter")]
    Counter,
    #[serde(rename = "cabinet")]
    Cabinet,
    #[serde(rename = "drawer")]
    Drawer,
    #[serde(rename = "shelf")]
    Shelf,
    #[serde(rename = "bin")]
    Bin,
    #[serde(rename = "door")]
    Door,
}

impl fmt::Display for LocationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocationType::Fridge => write!(f, "fridge"),
            LocationType::Freezer => write!(f, "freezer"),
            LocationType::Pantry => write!(f, "pantry"),
            LocationType::Counter => write!(f, "counter"),
            LocationType::Cabinet => write!(f, "cabinet"),
            LocationType::Drawer => write!(f, "drawer"),
            LocationType::Shelf => write!(f, "shelf"),
            LocationType::Bin => write!(f, "bin"),
            LocationType::Door => write!(f, "door"),
        }
    }
}

impl std::str::FromStr for LocationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fridge" => Ok(LocationType::Fridge),
            "freezer" => Ok(LocationType::Freezer),
            "pantry" => Ok(LocationType::Pantry),
            "counter" => Ok(LocationType::Counter),
            "cabinet" => Ok(LocationType::Cabinet),
            "drawer" => Ok(LocationType::Drawer),
            "shelf" => Ok(LocationType::Shelf),
            "bin" => Ok(LocationType::Bin),
            "door" => Ok(LocationType::Door),
            _ => Err(format!("Unknown location type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    #[serde(rename = "exact")]
    Exact,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "empty")]
    Empty,
}

impl fmt::Display for ConfidenceLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfidenceLevel::Exact => write!(f, "exact"),
            ConfidenceLevel::High => write!(f, "high"),
            ConfidenceLevel::Medium => write!(f, "medium"),
            ConfidenceLevel::Low => write!(f, "low"),
            ConfidenceLevel::Empty => write!(f, "empty"),
        }
    }
}

impl std::str::FromStr for ConfidenceLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "exact" => Ok(ConfidenceLevel::Exact),
            "high" => Ok(ConfidenceLevel::High),
            "medium" => Ok(ConfidenceLevel::Medium),
            "low" => Ok(ConfidenceLevel::Low),
            "empty" => Ok(ConfidenceLevel::Empty),
            _ => Err(format!("Unknown confidence level: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub location_id: Uuid,
    pub name: String,
    pub location_type: LocationType,
    pub parent_location_id: Option<Uuid>,
    pub temperature_controlled: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Location {
    pub async fn create(
        pool: &PgPool,
        name: String,
        location_type: LocationType,
        parent_location_id: Option<Uuid>,
        temperature_controlled: bool,
        notes: Option<String>,
    ) -> Result<Self> {
        let location = sqlx::query_as!(
            Location,
            r#"
            INSERT INTO locations (
                name, location_type, parent_location_id, 
                temperature_controlled, notes
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING 
                location_id,
                name,
                location_type as "location_type: LocationType",
                parent_location_id,
                temperature_controlled,
                notes,
                created_at,
                updated_at
            "#,
            name,
            location_type as LocationType,
            parent_location_id,
            temperature_controlled,
            notes
        )
        .fetch_one(pool)
        .await?;

        Ok(location)
    }

    pub async fn get_by_id(pool: &PgPool, location_id: Uuid) -> Result<Option<Self>> {
        let location = sqlx::query_as!(
            Location,
            r#"
            SELECT 
                location_id,
                name,
                location_type as "location_type: LocationType",
                parent_location_id,
                temperature_controlled,
                notes,
                created_at,
                updated_at
            FROM locations
            WHERE location_id = $1
            "#,
            location_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(location)
    }

    pub async fn get_by_type(pool: &PgPool, location_type: LocationType) -> Result<Vec<Self>> {
        let locations = sqlx::query_as!(
            Location,
            r#"
            SELECT 
                location_id,
                name,
                location_type as "location_type: LocationType",
                parent_location_id,
                temperature_controlled,
                notes,
                created_at,
                updated_at
            FROM locations
            WHERE location_type = $1
            "#,
            location_type as LocationType
        )
        .fetch_all(pool)
        .await?;

        Ok(locations)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub inventory_id: Uuid,
    pub ingredient_id: Uuid,
    pub location_id: Uuid,
    pub quantity: f64,
    pub unit_id: Uuid,
    pub expiration_date: Option<NaiveDate>,
    pub purchase_date: Option<NaiveDate>,
    pub confidence_level: ConfidenceLevel,
    pub minimum_quantity: Option<f64>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Inventory {
    pub async fn create(
        pool: &PgPool,
        ingredient_id: Uuid,
        location_id: Uuid,
        quantity: f64,
        unit_id: Uuid,
        expiration_date: Option<NaiveDate>,
        purchase_date: Option<NaiveDate>,
        confidence_level: ConfidenceLevel,
        minimum_quantity: Option<f64>,
        notes: Option<String>,
    ) -> Result<Self> {
        let inventory = sqlx::query_as!(
            Inventory,
            r#"
            INSERT INTO inventory (
                ingredient_id, location_id, quantity, unit_id,
                expiration_date, purchase_date, confidence_level,
                minimum_quantity, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING 
                inventory_id,
                ingredient_id,
                location_id,
                quantity,
                unit_id,
                expiration_date,
                purchase_date,
                confidence_level as "confidence_level: ConfidenceLevel",
                minimum_quantity,
                notes,
                created_at,
                updated_at
            "#,
            ingredient_id,
            location_id,
            quantity,
            unit_id,
            expiration_date,
            purchase_date,
            confidence_level as ConfidenceLevel,
            minimum_quantity,
            notes
        )
        .fetch_one(pool)
        .await?;

        Ok(inventory)
    }

    pub async fn update_quantity(
        &self,
        pool: &PgPool,
        quantity: f64,
        confidence_level: ConfidenceLevel,
    ) -> Result<Self> {
        let updated = sqlx::query_as!(
            Inventory,
            r#"
            UPDATE inventory
            SET quantity = $2,
                confidence_level = $3,
                updated_at = NOW()
            WHERE inventory_id = $1
            RETURNING 
                inventory_id,
                ingredient_id,
                location_id,
                quantity,
                unit_id,
                expiration_date,
                purchase_date,
                confidence_level as "confidence_level: ConfidenceLevel",
                minimum_quantity,
                notes,
                created_at,
                updated_at
            "#,
            self.inventory_id,
            quantity,
            confidence_level as ConfidenceLevel
        )
        .fetch_one(pool)
        .await?;

        Ok(updated)
    }

    pub async fn get_by_ingredient(pool: &PgPool, ingredient_id: Uuid) -> Result<Vec<Self>> {
        let inventory = sqlx::query_as!(
            Inventory,
            r#"
            SELECT 
                inventory_id,
                ingredient_id,
                location_id,
                quantity,
                unit_id,
                expiration_date,
                purchase_date,
                confidence_level as "confidence_level: ConfidenceLevel",
                minimum_quantity,
                notes,
                created_at,
                updated_at
            FROM inventory
            WHERE ingredient_id = $1
            ORDER BY expiration_date NULLS LAST
            "#,
            ingredient_id
        )
        .fetch_all(pool)
        .await?;

        Ok(inventory)
    }

    pub async fn get_low_items(pool: &PgPool) -> Result<Vec<Self>> {
        let inventory = sqlx::query_as!(
            Inventory,
            r#"
            SELECT 
                inventory_id,
                ingredient_id,
                location_id,
                quantity,
                unit_id,
                expiration_date,
                purchase_date,
                confidence_level as "confidence_level: ConfidenceLevel",
                minimum_quantity,
                notes,
                created_at,
                updated_at
            FROM inventory
            WHERE minimum_quantity IS NOT NULL
                AND quantity < minimum_quantity
                AND confidence_level != 'empty'
            ORDER BY (quantity / minimum_quantity)
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(inventory)
    }
}
