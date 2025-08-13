-- Reverse migration script - removes all cooking schema objects

-- Drop all triggers first
DROP TRIGGER IF EXISTS update_meal_plan_entries_updated_at ON meal_plan_entries;
DROP TRIGGER IF EXISTS update_meal_plans_updated_at ON meal_plans;
DROP TRIGGER IF EXISTS update_inventory_updated_at ON inventory;
DROP TRIGGER IF EXISTS update_locations_updated_at ON locations;
DROP TRIGGER IF EXISTS update_tags_updated_at ON tags;
DROP TRIGGER IF EXISTS update_recipe_steps_updated_at ON recipe_steps;
DROP TRIGGER IF EXISTS update_recipe_ingredients_updated_at ON recipe_ingredients;
DROP TRIGGER IF EXISTS update_equipment_updated_at ON equipment;
DROP TRIGGER IF EXISTS update_units_updated_at ON units;
DROP TRIGGER IF EXISTS update_ingredients_updated_at ON ingredients;
DROP TRIGGER IF EXISTS update_recipes_updated_at ON recipes;

-- Drop the trigger function
DROP FUNCTION IF EXISTS update_updated_at_column();

-- Drop all indexes
DROP INDEX IF EXISTS idx_recipe_variations_variation_id;
DROP INDEX IF EXISTS idx_recipe_variations_recipe_id;
DROP INDEX IF EXISTS idx_meal_plan_entries_date;
DROP INDEX IF EXISTS idx_meal_plan_entries_meal_plan_id;
DROP INDEX IF EXISTS idx_inventory_location_id;
DROP INDEX IF EXISTS idx_inventory_ingredient_id;
DROP INDEX IF EXISTS idx_step_equipment_step_id;
DROP INDEX IF EXISTS idx_step_ingredients_step_id;
DROP INDEX IF EXISTS idx_recipe_equipment_recipe_id;
DROP INDEX IF EXISTS idx_recipe_ingredients_recipe_id;
DROP INDEX IF EXISTS idx_recipe_steps_recipe_id;

-- Drop tables in reverse order of dependencies
DROP TABLE IF EXISTS meal_plan_entries;
DROP TABLE IF EXISTS meal_plans;
DROP TABLE IF EXISTS inventory;
DROP TABLE IF EXISTS locations;
DROP TABLE IF EXISTS recipe_tags;
DROP TABLE IF EXISTS tags;
DROP TABLE IF EXISTS step_equipment;
DROP TABLE IF EXISTS step_ingredients;
DROP TABLE IF EXISTS recipe_steps;
DROP TABLE IF EXISTS recipe_ingredients;
DROP TABLE IF EXISTS recipe_equipment;
DROP TABLE IF EXISTS equipment;
DROP TABLE IF EXISTS ingredients;
DROP TABLE IF EXISTS units;
DROP TABLE IF EXISTS recipe_variations;
DROP TABLE IF EXISTS recipes;