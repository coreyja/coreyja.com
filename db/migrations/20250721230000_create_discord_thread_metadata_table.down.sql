-- Re-add metadata column to threads table
ALTER TABLE threads ADD COLUMN metadata JSONB DEFAULT '{}'::jsonb;

-- Recreate the index
CREATE INDEX idx_threads_metadata_discord 
ON threads ((metadata->'discord'->>'thread_id'))
WHERE metadata->'discord'->>'thread_id' IS NOT NULL;

-- Drop the discord_thread_metadata table and its associated objects
DROP TRIGGER IF EXISTS update_discord_thread_metadata_updated_at ON discord_thread_metadata;
DROP FUNCTION IF EXISTS update_discord_thread_metadata_updated_at();
DROP TABLE IF EXISTS discord_thread_metadata;