-- Add thread_type column with default value
ALTER TABLE threads 
ADD COLUMN thread_type TEXT DEFAULT 'goal_oriented' NOT NULL
CHECK (thread_type IN ('goal_oriented', 'interactive'));

-- Add metadata column for storing Discord and other metadata
ALTER TABLE threads 
ADD COLUMN metadata JSONB DEFAULT '{}'::jsonb;

-- Create index for Discord thread lookups
CREATE INDEX idx_threads_metadata_discord 
ON threads ((metadata->'discord'->>'thread_id'))
WHERE metadata->'discord'->>'thread_id' IS NOT NULL;

-- Add new stitch type for Discord messages
-- First, we need to drop the existing constraint
ALTER TABLE stitches 
DROP CONSTRAINT IF EXISTS stitches_stitch_type_check;

-- Add the new constraint with Discord message stitch type
ALTER TABLE stitches 
ADD CONSTRAINT stitches_stitch_type_check 
CHECK (stitch_type IN ('initial_prompt', 'llm_call', 'tool_call', 'thread_result', 'discord_message'));
