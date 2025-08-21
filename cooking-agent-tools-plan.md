# Cooking Schema AI Agent Tools Plan

## Overview
This document outlines a minimal set of tools for an AI agent to interact with the cooking database schema. The tools are designed to cover the most common operations while keeping complexity low.

## Core Tools

### 1. Recipe Management
- **UpsertRecipe** - Create or update a recipe with full details including steps, ingredients, and equipment
- **GetRecipe** - Retrieve a recipe by ID or name with all details

### 2. Inventory Management
- **UpdateInventory** - Add/update ingredient quantities in inventory
- **CheckInventory** - Check what ingredients are available and their quantities

### 3. Meal Planning
- **CreateMealPlan** - Create a meal plan for a date range
- **AddRecipeToMealPlan** - Schedule recipes for specific meals
- **ListMealPlans** - List current and upcoming meal plans

## Tool Details

### UpsertRecipe
**Parameters:**
- recipe_id (optional - if provided, updates existing recipe)
- name (required)
- description
- prep_time (minutes)
- cook_time (minutes)
- servings (required)
- author_user_id (required)
- ingredients (array):
  - ingredient_name (required) - will create if doesn't exist
  - quantity (required)
  - unit_name (required) - will create if doesn't exist
  - is_optional
  - notes
  - ingredient_group (e.g., "For the sauce")
  - preparation (diced/minced/chopped/etc)
  - temperature (room_temp/chilled/frozen/etc)
- steps (array):
  - step_number (required)
  - instruction (required)
  - duration (minutes)
  - temperature
  - temperature_unit (F/C)
- equipment (array):
  - equipment_name (required) - will create if doesn't exist
  - is_optional
  - notes
- tags (array of tag names) - will create if don't exist

**Returns:** recipe_id

**Behavior:**
- If recipe_id is provided, updates the existing recipe (replacing all ingredients, steps, equipment)
- If recipe_id is not provided, creates a new recipe
- Automatically creates any missing ingredients, units, equipment, or tags

### GetRecipe
**Parameters:**
- recipe_id or name (one required)

**Returns:** Complete recipe with ingredients, steps, equipment, and tags

### UpdateInventory
**Parameters:**
- ingredient_name (required) - will create if doesn't exist
- quantity (required)
- unit_name (required) - will create if doesn't exist
- confidence_level (exact/high/medium/low/empty)
- location_name - will create if doesn't exist
- expiration_date

### CheckInventory
**Parameters:**
- ingredient_names (array) - if empty, returns all

**Returns:** Available quantities and locations

### ListMealPlans
**Parameters:**
- start_date (optional) - defaults to today
- end_date (optional) - defaults to 7 days from start_date
- include_past (boolean) - defaults to false

**Returns:** List of meal plans with their scheduled recipes

## Design Principles

1. **Auto-creation**: Tools should create missing entities (ingredients, units) rather than requiring separate creation steps
2. **Name-based lookup**: Use names instead of IDs where possible for ease of use
3. **Sensible defaults**: Optional fields have reasonable defaults
4. **Atomic operations**: Each tool does one thing well
5. **Read-heavy**: Optimize for reading/searching over complex updates

## Future Considerations

These tools could be extended later for:
- Recipe variations and forking
- Tag management
- Equipment requirements per step
- Shopping list generation from meal plans
- Inventory depletion from cooking

## Implementation Notes

- All tools should handle database transactions properly
- Name lookups should be case-insensitive
- Tools should validate data (e.g., step numbers are sequential)
- Error messages should be clear and actionable