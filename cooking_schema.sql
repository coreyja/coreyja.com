-- Core recipe tables
CREATE TABLE
  recipes (
    recipe_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    name TEXT NOT NULL,
    description TEXT,
    prep_time INTEGER, -- minutes
    cook_time INTEGER, -- minutes
    servings INTEGER NOT NULL,
    -- Recipe scaling columns
    yield_amount DECIMAL,
    yield_unit TEXT, -- e.g., "cookies", "loaves", "cups"
    -- Author tracking
    author_user_id UUID NOT NULL, -- references users table in your main schema
    generated_by_stitch UUID NULL REFERENCES stitches (stitch_id),
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

-- Units table
CREATE TABLE
  units (
    unit_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    name TEXT NOT NULL UNIQUE,
    type TEXT CHECK (type IN ('volume', 'weight', 'count')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

-- Ingredients table (fixed with proper FK)
CREATE TABLE
  ingredients (
    ingredient_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    name TEXT NOT NULL UNIQUE,
    category TEXT,
    default_unit_id UUID REFERENCES units (unit_id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

-- Equipment table
CREATE TABLE
  equipment (
    equipment_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    name TEXT NOT NULL UNIQUE,
    category TEXT CHECK (
      category IN (
        'cookware',
        'bakeware',
        'appliance',
        'tool',
        'utensil',
        'measuring',
        'mixing',
        'cutting'
      )
    ),
    is_optional BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

-- Recipe equipment junction table
CREATE TABLE
  recipe_equipment (
    recipe_id UUID REFERENCES recipes (recipe_id) ON DELETE CASCADE,
    equipment_id UUID REFERENCES equipment (equipment_id) ON DELETE CASCADE,
    is_optional BOOLEAN DEFAULT false,
    notes TEXT,
    PRIMARY KEY (recipe_id, equipment_id)
  );

-- Recipe ingredients with unique constraint
CREATE TABLE
  recipe_ingredients (
    recipe_ingredient_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    recipe_id UUID REFERENCES recipes (recipe_id) ON DELETE CASCADE,
    ingredient_id UUID REFERENCES ingredients (ingredient_id) ON DELETE RESTRICT,
    quantity DECIMAL NOT NULL,
    unit_id UUID REFERENCES units (unit_id) ON DELETE RESTRICT,
    display_order INTEGER,
    preparation TEXT CHECK (
      preparation IN (
        'diced',
        'minced',
        'chopped',
        'sliced',
        'julienned',
        'grated',
        'zested',
        'crushed',
        'mashed',
        'whole',
        'halved',
        'quartered',
        NULL
      )
    ),
    temperature TEXT CHECK (
      temperature IN (
        'room_temp',
        'chilled',
        'frozen',
        'melted',
        'softened',
        NULL
      )
    ),
    is_optional BOOLEAN DEFAULT false,
    notes TEXT, -- e.g., "or 2 cups frozen"
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Prevent duplicate ingredients in same recipe
    UNIQUE (recipe_id, ingredient_id)
  );

-- Recipe steps (normalized instructions)
CREATE TABLE
  recipe_steps (
    step_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    recipe_id UUID REFERENCES recipes (recipe_id) ON DELETE CASCADE,
    step_number INTEGER NOT NULL,
    instruction TEXT NOT NULL,
    duration INTEGER, -- minutes
    temperature INTEGER, -- degrees (if baking/cooking temp needed)
    temperature_unit TEXT CHECK (temperature_unit IN ('F', 'C')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (recipe_id, step_number)
  );

-- Link ingredients to specific steps
CREATE TABLE
  step_ingredients (
    step_id UUID REFERENCES recipe_steps (step_id) ON DELETE CASCADE,
    recipe_ingredient_id UUID REFERENCES recipe_ingredients (recipe_ingredient_id) ON DELETE CASCADE,
    PRIMARY KEY (step_id, recipe_ingredient_id)
  );

-- Link equipment to specific steps
CREATE TABLE
  step_equipment (
    step_id UUID REFERENCES recipe_steps (step_id) ON DELETE CASCADE,
    equipment_id UUID REFERENCES equipment (equipment_id) ON DELETE CASCADE,
    PRIMARY KEY (step_id, equipment_id)
  );

-- Tags
CREATE TABLE
  tags (
    tag_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    name TEXT NOT NULL UNIQUE,
    color TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE
  recipe_tags (
    recipe_id UUID REFERENCES recipes (recipe_id) ON DELETE CASCADE,
    tag_id UUID REFERENCES tags (tag_id) ON DELETE CASCADE,
    PRIMARY KEY (recipe_id, tag_id)
  );

-- Locations
CREATE TABLE
  locations (
    location_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    name TEXT NOT NULL,
    parent_id UUID REFERENCES locations (location_id) ON DELETE CASCADE,
    location_type TEXT CHECK (
      location_type IN (
        'fridge',
        'freezer',
        'pantry',
        'counter',
        'cabinet',
        'drawer',
        'shelf',
        'bin',
        'door'
      )
    ),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

-- Inventory
CREATE TABLE
  inventory (
    inventory_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    ingredient_id UUID REFERENCES ingredients (ingredient_id) ON DELETE CASCADE,
    quantity DECIMAL NOT NULL,
    unit_id UUID REFERENCES units (unit_id) ON DELETE RESTRICT,
    expiration_date DATE,
    location_id UUID REFERENCES locations (location_id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

-- Meal Planning
CREATE TABLE
  meal_plans (
    meal_plan_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    name TEXT NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

CREATE TABLE
  meal_plan_entries (
    meal_plan_entry_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    meal_plan_id UUID REFERENCES meal_plans (meal_plan_id) ON DELETE CASCADE,
    recipe_id UUID REFERENCES recipes (recipe_id) ON DELETE CASCADE,
    date DATE NOT NULL,
    meal_type TEXT CHECK (
      meal_type IN ('breakfast', 'lunch', 'dinner', 'snack')
    ),
    servings_override INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );

-- Indexes for performance
CREATE INDEX idx_recipe_steps_recipe_id ON recipe_steps (recipe_id);
CREATE INDEX idx_recipe_ingredients_recipe_id ON recipe_ingredients (recipe_id);
CREATE INDEX idx_recipe_equipment_recipe_id ON recipe_equipment (recipe_id);
CREATE INDEX idx_step_ingredients_step_id ON step_ingredients (step_id);
CREATE INDEX idx_step_equipment_step_id ON step_equipment (step_id);
CREATE INDEX idx_inventory_ingredient_id ON inventory (ingredient_id);
CREATE INDEX idx_inventory_location_id ON inventory (location_id);
CREATE INDEX idx_meal_plan_entries_meal_plan_id ON meal_plan_entries (meal_plan_id);
CREATE INDEX idx_meal_plan_entries_date ON meal_plan_entries (date);

-- Trigger to update updated_at timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply update trigger to all tables with updated_at
CREATE TRIGGER update_recipes_updated_at BEFORE UPDATE ON recipes
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_ingredients_updated_at BEFORE UPDATE ON ingredients
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_units_updated_at BEFORE UPDATE ON units
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_equipment_updated_at BEFORE UPDATE ON equipment
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_recipe_ingredients_updated_at BEFORE UPDATE ON recipe_ingredients
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_recipe_steps_updated_at BEFORE UPDATE ON recipe_steps
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_tags_updated_at BEFORE UPDATE ON tags
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_locations_updated_at BEFORE UPDATE ON locations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_inventory_updated_at BEFORE UPDATE ON inventory
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_meal_plans_updated_at BEFORE UPDATE ON meal_plans
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_meal_plan_entries_updated_at BEFORE UPDATE ON meal_plan_entries
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();