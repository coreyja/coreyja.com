use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    al::tools::{ThreadContext, Tool},
    AppState,
};

// CheckInventory Tool
#[derive(Clone, Debug)]
pub struct CheckInventory;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CheckInventoryInput {
    pub ingredient_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InventoryItem {
    pub ingredient_name: String,
    pub quantity: f64,
    pub unit_name: String,
    pub confidence_level: String,
    pub location_name: Option<String>,
    pub expiration_date: Option<String>,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CheckInventoryOutput {
    pub inventory: Vec<InventoryItem>,
}

#[async_trait::async_trait]
impl Tool for CheckInventory {
    const NAME: &'static str = "check_inventory";
    const DESCRIPTION: &'static str = r#"
    Check what ingredients are available and their quantities.

    If ingredient_names is empty, returns all inventory items.

    Example for specific items:
    ```json
    {
        "ingredient_names": ["flour", "sugar", "eggs"]
    }
    ```

    Example for all items:
    ```json
    {
        "ingredient_names": []
    }
    ```
    "#;

    type ToolInput = CheckInventoryInput;
    type ToolOutput = CheckInventoryOutput;

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        _context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        let pool = &app_state.db;

        // Build ingredient filter
        let ingredient_filter = if input.ingredient_names.is_empty() {
            None
        } else {
            Some(input.ingredient_names.clone())
        };

        // Get inventory items with optional filter
        let inventory_items = sqlx::query!(
            r#"
            SELECT
                i.inventory_id,
                ing.name as ingredient_name,
                i.quantity,
                i.unit_id,
                i.confidence_level,
                i.location_id,
                i.expiration_date,
                i.updated_at
            FROM inventory i
            JOIN ingredients ing ON i.ingredient_id = ing.ingredient_id
            WHERE ($1::text[] IS NULL OR LOWER(ing.name) = ANY(
                SELECT LOWER(unnest($1::text[]))
            ))
            ORDER BY ing.name
            "#,
            ingredient_filter.as_deref()
        )
        .fetch_all(pool)
        .await?;

        let mut inventory = Vec::new();

        for item in inventory_items {
            // Get unit name if unit_id exists
            let unit_name = if let Some(unit_id) = item.unit_id {
                sqlx::query!("SELECT name FROM units WHERE unit_id = $1", unit_id)
                    .fetch_optional(pool)
                    .await?
                    .map_or_else(|| "units".to_string(), |u| u.name)
            } else {
                "units".to_string()
            };

            // Get location name if location_id exists
            let location_name = if let Some(location_id) = item.location_id {
                sqlx::query!(
                    "SELECT name FROM locations WHERE location_id = $1",
                    location_id
                )
                .fetch_optional(pool)
                .await?
                .map(|l| l.name)
            } else {
                None
            };

            inventory.push(InventoryItem {
                ingredient_name: item.ingredient_name,
                quantity: item.quantity.to_string().parse::<f64>()?,
                unit_name,
                confidence_level: item.confidence_level.unwrap_or("medium".to_string()),
                location_name,
                expiration_date: item.expiration_date.map(|d| d.to_string()),
                last_updated: item.updated_at.to_string(),
            });
        }

        Ok(CheckInventoryOutput { inventory })
    }
}
