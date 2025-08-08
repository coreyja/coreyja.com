-- Add mode column to threads table
ALTER TABLE threads ADD COLUMN mode text;

-- Add check constraint for valid modes
ALTER TABLE threads ADD CONSTRAINT threads_mode_check 
    CHECK (mode IN ('general', 'cooking', 'project_manager'));

-- Update existing threads to have 'general' mode
UPDATE threads SET mode = 'general' WHERE mode IS NULL;

-- Make mode NOT NULL with default
ALTER TABLE threads ALTER COLUMN mode SET NOT NULL;
ALTER TABLE threads ALTER COLUMN mode SET DEFAULT 'general';