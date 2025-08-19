use bigdecimal::BigDecimal;
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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Location {
    pub location_id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub location_type: Option<LocationType>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Location {
    pub async fn create(
        pool: &PgPool,
        name: String,
        parent_id: Option<Uuid>,
        location_type: Option<LocationType>,
    ) -> Result<Self> {
        let location_type_str = location_type.as_ref().map(std::string::ToString::to_string);

        let location = sqlx::query_as!(
            Location,
            r#"
            INSERT INTO locations (
                name,
                parent_id,
                location_type
            )
            VALUES ($1, $2, $3)
            RETURNING
                location_id as "location_id!",
                name,
                parent_id,
                location_type as "location_type: LocationType",
                created_at,
                updated_at
            "#,
            name,
            parent_id,
            location_type_str
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
                location_id as "location_id!",
                name,
                parent_id,
                location_type as "location_type: LocationType",
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

    pub async fn get_by_type(
        pool: &PgPool,
        location_type: Option<LocationType>,
    ) -> Result<Vec<Self>> {
        let location_type_str = location_type.as_ref().map(std::string::ToString::to_string);

        let locations = sqlx::query_as!(
            Location,
            r#"
            SELECT
                location_id as "location_id!",
                name,
                parent_id,
                location_type as "location_type: LocationType",
                created_at,
                updated_at
            FROM locations
            WHERE $1::TEXT IS NULL OR location_type = $1
            "#,
            location_type_str
        )
        .fetch_all(pool)
        .await?;

        Ok(locations)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Inventory {
    pub inventory_id: Uuid,
    pub ingredient_id: Uuid,
    pub quantity: BigDecimal,
    pub unit_id: Option<Uuid>,
    pub confidence_level: Option<ConfidenceLevel>,
    pub expiration_date: Option<NaiveDate>,
    pub location_id: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Inventory {
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        pool: &PgPool,
        ingredient_id: Uuid,
        quantity: BigDecimal,
        unit_id: Option<Uuid>,
        confidence_level: Option<ConfidenceLevel>,
        expiration_date: Option<NaiveDate>,
        location_id: Option<Uuid>,
        notes: Option<String>,
    ) -> Result<Self> {
        let confidence_level_str = confidence_level
            .as_ref()
            .map(std::string::ToString::to_string);

        let inventory = sqlx::query_as!(
            Inventory,
            r#"
            INSERT INTO inventory (
                ingredient_id,
                quantity,
                unit_id,
                confidence_level,
                expiration_date,
                location_id,
                notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING
                inventory_id as "inventory_id!",
                ingredient_id as "ingredient_id!",
                quantity,
                unit_id as "unit_id!",
                confidence_level as "confidence_level: ConfidenceLevel",
                expiration_date,
                location_id as "location_id!",
                notes,
                created_at,
                updated_at
            "#,
            ingredient_id,
            quantity,
            unit_id,
            confidence_level_str,
            expiration_date,
            location_id,
            notes
        )
        .fetch_one(pool)
        .await?;

        Ok(inventory)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        pool: &PgPool,
        quantity: Option<BigDecimal>,
        unit_id: Option<Uuid>,
        confidence_level: Option<ConfidenceLevel>,
        expiration_date: Option<NaiveDate>,
        location_id: Option<Uuid>,
        notes: Option<String>,
    ) -> Result<Self> {
        let confidence_level_str = confidence_level
            .as_ref()
            .map(std::string::ToString::to_string);

        let updated = sqlx::query_as!(
            Inventory,
            r#"
            UPDATE inventory
            SET quantity = COALESCE($2, quantity),
                unit_id = COALESCE($3, unit_id),
                confidence_level = COALESCE($4::TEXT, confidence_level),
                expiration_date = COALESCE($5, expiration_date),
                location_id = COALESCE($6, location_id),
                notes = COALESCE($7, notes)
            WHERE inventory_id = $1
            RETURNING
                inventory_id as "inventory_id!",
                ingredient_id as "ingredient_id!",
                quantity,
                unit_id as "unit_id!",
                confidence_level as "confidence_level: ConfidenceLevel",
                expiration_date,
                location_id as "location_id!",
                notes,
                created_at,
                updated_at
            "#,
            self.inventory_id,
            quantity,
            unit_id,
            confidence_level_str,
            expiration_date,
            location_id,
            notes
        )
        .fetch_one(pool)
        .await?;

        Ok(updated)
    }

    pub async fn get_by_location(pool: &PgPool, location_id: Uuid) -> Result<Vec<Self>> {
        let inventory = sqlx::query_as!(
            Inventory,
            r#"
            SELECT
                inventory_id as "inventory_id!",
                ingredient_id as "ingredient_id!",
                quantity,
                unit_id as "unit_id!",
                confidence_level as "confidence_level: ConfidenceLevel",
                expiration_date,
                location_id as "location_id!",
                notes,
                created_at,
                updated_at
            FROM inventory
            WHERE location_id = $1
            "#,
            location_id
        )
        .fetch_all(pool)
        .await?;

        Ok(inventory)
    }

    pub async fn get_by_ingredient(pool: &PgPool, ingredient_id: Uuid) -> Result<Vec<Self>> {
        let inventory = sqlx::query_as!(
            Inventory,
            r#"
            SELECT
                inventory_id as "inventory_id!",
                ingredient_id as "ingredient_id!",
                quantity,
                unit_id as "unit_id!",
                confidence_level as "confidence_level: ConfidenceLevel",
                expiration_date,
                location_id as "location_id!",
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

    pub async fn get_expiring_soon(pool: &PgPool, days: i32) -> Result<Vec<Self>> {
        let inventory = sqlx::query_as!(
            Inventory,
            r#"
            SELECT
                inventory_id as "inventory_id!",
                ingredient_id as "ingredient_id!",
                quantity,
                unit_id as "unit_id!",
                confidence_level as "confidence_level: ConfidenceLevel",
                expiration_date,
                location_id as "location_id!",
                notes,
                created_at,
                updated_at
            FROM inventory
            WHERE expiration_date IS NOT NULL
                AND expiration_date <= CURRENT_DATE + INTERVAL '1 day' * $1
            ORDER BY expiration_date
            "#,
            f64::from(days)
        )
        .fetch_all(pool)
        .await?;

        Ok(inventory)
    }
}
