-- Add agent_name column to threads table to track which agent handles each thread
ALTER TABLE threads ADD COLUMN agent_name text;

-- Set default agent_name 'Al' for existing threads with NULL agent_name
UPDATE threads SET agent_name = 'Al' WHERE agent_name IS NULL;

-- Make agent_name column NOT NULL with default value
ALTER TABLE threads ALTER COLUMN agent_name SET NOT NULL;

-- Create index for agent_name lookups
CREATE INDEX idx_threads_agent_name ON threads(agent_name);
