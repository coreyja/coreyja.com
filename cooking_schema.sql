-- Core recipe tables
CREATE TABLE
  recipes (
    recipe_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    name TEXT NOT NULL,
    description TEXT,
    instructions JSONB NOT NULL CHECK (jsonb_typeof (instructions) = 'array'),
    prep_time INTEGER, -- minutes
    cook_time INTEGER, -- minutes
    servings INTEGER NOT NULL,
    generated_by_model TEXT -- NULL for human recipes, 'gpt-4', 'claude-3', etc for AI
  );

CREATE TABLE
  ingredients (
    ingredient_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    name TEXT NOT NULL UNIQUE,
    category TEXT,
    default_unit_id UUID
  );

CREATE TABLE
  units (
    unit_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    name TEXT NOT NULL UNIQUE,
    type TEXT CHECK (type IN ('volume', 'weight', 'count'))
  );

CREATE TABLE
  recipe_ingredients (
    recipe_ingredient_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    recipe_id UUID REFERENCES recipes (recipe_id),
    ingredient_id UUID REFERENCES ingredients (ingredient_id),
    quantity DECIMAL NOT NULL,
    unit_id UUID REFERENCES units (unit_id),
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
    is_used_in_multiple_steps BOOLEAN DEFAULT false
  );

-- Tags
CREATE TABLE
  tags (
    tag_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    name TEXT NOT NULL UNIQUE,
    color TEXT
  );

CREATE TABLE
  recipe_tags (
    recipe_id UUID REFERENCES recipes (recipe_id),
    tag_id UUID REFERENCES tags (tag_id),
    PRIMARY KEY (recipe_id, tag_id)
  );

-- Locations
CREATE TABLE
  locations (
    location_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    name TEXT NOT NULL,
    parent_id UUID REFERENCES locations (location_id),
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
    )
  );

-- Inventory
CREATE TABLE
  inventory (
    inventory_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    ingredient_id UUID REFERENCES ingredients (ingredient_id),
    quantity DECIMAL NOT NULL,
    unit_id UUID REFERENCES units (unit_id),
    expiration_date DATE,
    location_id UUID REFERENCES locations (location_id)
  );

-- Meal Planning
CREATE TABLE
  meal_plans (
    meal_plan_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    name TEXT NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    notes TEXT
  );

CREATE TABLE
  meal_plan_entries (
    meal_plan_entry_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    meal_plan_id UUID REFERENCES meal_plans (meal_plan_id),
    recipe_id UUID REFERENCES recipes (recipe_id),
    date DATE NOT NULL,
    meal_type TEXT CHECK (
      meal_type IN ('breakfast', 'lunch', 'dinner', 'snack')
    ),
    servings_override INTEGER
  );
