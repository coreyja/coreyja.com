-- Drop the tool_suggestions table and related objects
DROP TRIGGER IF EXISTS update_tool_suggestions_updated_at ON tool_suggestions;
DROP FUNCTION IF EXISTS update_tool_suggestions_updated_at();
DROP TABLE IF EXISTS tool_suggestions;