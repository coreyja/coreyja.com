use std::str::FromStr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    al::tools::{ThreadContext, Tool},
    AppState,
};

// UpdateInventory Tool
#[derive(Clone, Debug)]
pub struct UpdateInventory;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateInventoryInput {
    pub ingredient_name: String,
    pub quantity: f64,
    pub unit_name: String,
    pub confidence_level: Option<String>,
    pub location_name: Option<String>,
    pub expiration_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateInventoryOutput {
    pub inventory_id: String,
    pub message: String,
}

#[async_trait::async_trait]
impl Tool for UpdateInventory {
    const NAME: &'static str = "update_inventory";
    const DESCRIPTION: &'static str = r#"
    Add or update ingredient quantities in inventory.

    Will create ingredient, unit, or location if they don't exist.
    Confidence levels: exact, high, medium, low, empty

    Example:
    ```json
    {
        "ingredient_name": "all-purpose flour",
        "quantity": 5,
        "unit_name": "pounds",
        "confidence_level": "high",
        "location_name": "Pantry Shelf 2",
        "expiration_date": "2024-12-31T00:00:00Z"
    }
    ```
    "#;

    type ToolInput = UpdateInventoryInput;
    type ToolOutput = UpdateInventoryOutput;

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        _context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        let pool = &app_state.db;

        // Get or create ingredient
        let ingredient =
            match db::cooking::Ingredient::get_by_name(pool, &input.ingredient_name).await? {
                Some(i) => i,
                None => {
                    db::cooking::Ingredient::create(pool, input.ingredient_name.clone(), None, None)
                        .await?
                }
            };

        // Get or create unit
        let unit = match db::cooking::Unit::get_by_name(pool, &input.unit_name).await? {
            Some(u) => u,
            None => {
                // Create unit with default type
                sqlx::query_as!(
                    db::cooking::Unit,
                    r#"
                    INSERT INTO units (unit_id, name, type, created_at, updated_at)
                    VALUES ($1, $2, $3, NOW(), NOW())
                    RETURNING unit_id, name, type as "unit_type: db::cooking::UnitType", created_at, updated_at
                    "#,
                    Uuid::new_v4(),
                    input.unit_name,
                    Some("volume")
                )
                .fetch_one(pool)
                .await?
            }
        };

        // Get or create location
        let location_id = if let Some(location_name) = input.location_name {
            // Try to find existing location by name
            let existing = sqlx::query!(
                "SELECT location_id FROM locations WHERE name = $1 LIMIT 1",
                location_name
            )
            .fetch_optional(pool)
            .await?;

            if let Some(loc) = existing {
                Some(loc.location_id)
            } else {
                // Create new location with pantry as default type
                let location = db::cooking::Location::create(
                    pool,
                    location_name,
                    None,
                    Some(db::cooking::LocationType::Pantry),
                )
                .await?;
                Some(location.location_id)
            }
        } else {
            None
        };

        // Parse confidence level
        let confidence = input
            .confidence_level
            .as_ref()
            .and_then(|c| db::cooking::ConfidenceLevel::from_str(c).ok())
            .unwrap_or(db::cooking::ConfidenceLevel::Medium);

        // Parse expiration date
        let expiration_date = input.expiration_date.as_ref().and_then(|d| {
            chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .or_else(|_| {
                    chrono::NaiveDateTime::parse_from_str(d, "%Y-%m-%dT%H:%M:%SZ")
                        .map(|dt| dt.date())
                })
                .ok()
        });

        // Convert quantity to BigDecimal
        let quantity = sqlx::types::BigDecimal::from_str(&input.quantity.to_string())?;

        // Check if inventory exists for this ingredient
        let existing_inventory =
            db::cooking::Inventory::get_by_ingredient(pool, ingredient.ingredient_id).await?;

        let inventory = if let Some(inv) = existing_inventory.into_iter().next() {
            // Update existing inventory
            inv.update(
                pool,
                Some(quantity),
                Some(unit.unit_id),
                Some(confidence),
                expiration_date,
                location_id,
                None, // notes
            )
            .await?
        } else {
            // Create new inventory
            db::cooking::Inventory::create(
                pool,
                ingredient.ingredient_id,
                quantity,
                Some(unit.unit_id),
                Some(confidence),
                expiration_date,
                location_id,
                None, // notes
            )
            .await?
        };

        Ok(UpdateInventoryOutput {
            inventory_id: inventory.inventory_id.to_string(),
            message: format!(
                "Updated inventory for {} with {} {}",
                input.ingredient_name, input.quantity, input.unit_name
            ),
        })
    }
}
