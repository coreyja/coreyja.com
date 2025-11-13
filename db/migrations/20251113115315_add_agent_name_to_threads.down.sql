-- Drop the index first
DROP INDEX IF EXISTS idx_threads_agent_name;

-- Remove agent_name column from threads table
ALTER TABLE threads DROP COLUMN agent_name;
