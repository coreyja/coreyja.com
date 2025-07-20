-- Drop index first
DROP INDEX IF EXISTS idx_tool_suggestions_previous_stitch_id;

-- Remove previous_stitch_id column from tool_suggestions table
ALTER TABLE tool_suggestions
DROP COLUMN previous_stitch_id;