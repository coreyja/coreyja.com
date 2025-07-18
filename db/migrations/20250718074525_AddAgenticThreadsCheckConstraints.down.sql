-- Remove check constraints added in the up migration

-- Remove threads table constraint
ALTER TABLE threads DROP CONSTRAINT IF EXISTS check_result_status;

-- Remove stitches table constraints
ALTER TABLE stitches DROP CONSTRAINT IF EXISTS check_llm_call_fields;
ALTER TABLE stitches DROP CONSTRAINT IF EXISTS check_tool_call_fields;
ALTER TABLE stitches DROP CONSTRAINT IF EXISTS check_thread_result_fields;