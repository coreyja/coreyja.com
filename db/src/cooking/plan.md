# Cooking Schema Implementation Plan

## Overview
Implement Rust models for the cooking/recipe database schema with appropriate enums for check constraints, query methods, and save operations.

## Architecture Patterns (Based on Existing Code)
- Use `sqlx::Type` with `#[sqlx(type_name = "text")]` for enums stored as text
- Include both `serde` and `sqlx` derive macros
- Use `#[serde(rename = "...")]` for enum variants
- Implement `Display` and `FromStr` traits for enums
- Use `sqlx::query_as!` macro for database queries
- Return `color_eyre::Result` from async methods
- Group related save operations (e.g., save all recipe steps together)

## Module Structure
```
db/src/cooking/
├── mod.rs           # Module exports and re-exports
├── recipe.rs        # Recipe, RecipeVariation models
├── ingredients.rs   # Unit (with UnitType enum), Ingredient, RecipeIngredient (with Preparation/Temperature enums)
├── equipment.rs     # Equipment (with EquipmentCategory enum), RecipeEquipment models
├── steps.rs         # RecipeStep (with TemperatureUnit enum), StepIngredient, StepEquipment models
├── inventory.rs     # Location (with LocationType enum), Inventory (with ConfidenceLevel enum)
├── meal_planning.rs # MealPlan, MealPlanEntry (with MealType enum)
└── tags.rs          # Tag, RecipeTag models
```

## Enums (Placed with their Models)

### ingredients.rs
- **UnitType**: `volume`, `weight`, `count` (for `units` table)
- **IngredientPreparation**: `diced`, `minced`, `chopped`, `sliced`, `julienned`, `grated`, `zested`, `crushed`, `mashed`, `whole`, `halved`, `quartered`, plus `None` variant (for `recipe_ingredients`)
- **IngredientTemperature**: `room_temp`, `chilled`, `frozen`, `melted`, `softened`, plus `None` variant (for `recipe_ingredients`)

### equipment.rs  
- **EquipmentCategory**: `cookware`, `bakeware`, `appliance`, `tool`, `utensil`, `measuring`, `mixing`, `cutting` (for `equipment` table)

### steps.rs
- **TemperatureUnit**: `F`, `C` (for `recipe_steps` table)

### inventory.rs
- **LocationType**: `fridge`, `freezer`, `pantry`, `counter`, `cabinet`, `drawer`, `shelf`, `bin`, `door` (for `locations` table)
- **ConfidenceLevel**: `exact`, `high`, `medium`, `low`, `empty` (for `inventory` table)

### meal_planning.rs
- **MealType**: `breakfast`, `lunch`, `dinner`, `snack` (for `meal_plan_entries` table)

## Core Models

### Recipe
- Fields: All from schema
- Methods:
  - `create()` - Create new recipe
  - `get_by_id()` - Fetch by ID
  - `list_by_author()` - List user's recipes
  - `update()` - Update basic fields
  - `get_full()` - Get with all related data (ingredients, steps, equipment)

### RecipeIngredient
- Methods:
  - `create()` - Create individual ingredient
  - `update()` - Update individual ingredient
  - `delete()` - Remove ingredient from recipe
  - `get_by_recipe()` - Get all ingredients for a recipe

### RecipeStep  
- No individual save - saved as collection
- Method: `save_all_for_recipe()` - Save all steps with proper ordering
- Method: `get_by_recipe()` - Get ordered steps for recipe

### Inventory
- Methods:
  - `create()` - Add inventory item
  - `update_quantity()` - Update quantity/confidence
  - `get_by_ingredient()` - Get inventory for ingredient
  - `get_low_items()` - Query items below threshold

### MealPlan
- Methods:
  - `create()` - Create plan with date range
  - `add_entry()` - Add recipe to plan
  - `get_by_date_range()` - Query plans

## Implementation Notes

1. **Transaction Handling**: Use database transactions for operations that modify multiple related tables (e.g., saving recipe with ingredients and steps)

2. **Cascade Deletes**: Leverage database cascade deletes where defined in schema

3. **Validation**: Add input validation in Rust code before database operations

4. **Error Handling**: Use `color_eyre::Result` consistently

5. **Testing**: Create unit tests for enum conversions and integration tests for database operations

## Next Steps
1. Create the cooking module structure
2. Implement enums with proper derives and trait implementations
3. Create model structs matching database schema
4. Implement query and save methods
5. Add validation and error handling
6. Write tests