-- Remove all constraints and indexes added in the up migration

-- Drop Discord metadata constraints
ALTER TABLE discord_thread_metadata DROP CONSTRAINT IF EXISTS check_participants_is_array;
ALTER TABLE discord_thread_metadata DROP CONSTRAINT IF EXISTS check_webhook_url_format;
ALTER TABLE discord_thread_metadata DROP CONSTRAINT IF EXISTS check_last_message_id_format;
ALTER TABLE discord_thread_metadata DROP CONSTRAINT IF EXISTS check_guild_id_format;
ALTER TABLE discord_thread_metadata DROP CONSTRAINT IF EXISTS check_channel_id_format;
ALTER TABLE discord_thread_metadata DROP CONSTRAINT IF EXISTS check_discord_thread_id_format;

-- Drop Discord metadata indexes
DROP INDEX IF EXISTS idx_discord_guild_created;

-- Drop stitches constraints
ALTER TABLE stitches DROP CONSTRAINT IF EXISTS check_no_self_reference;
ALTER TABLE stitches DROP CONSTRAINT IF EXISTS check_discord_message_fields;
ALTER TABLE stitches DROP CONSTRAINT IF EXISTS check_thread_result_fields;
ALTER TABLE stitches DROP CONSTRAINT IF EXISTS check_tool_call_fields;
ALTER TABLE stitches DROP CONSTRAINT IF EXISTS check_llm_call_fields;

-- Drop stitches indexes
DROP INDEX IF EXISTS idx_stitches_child_thread;
DROP INDEX IF EXISTS idx_stitches_thread_created;

-- Drop threads constraints
ALTER TABLE threads DROP CONSTRAINT IF EXISTS check_no_self_parent;
ALTER TABLE threads DROP CONSTRAINT IF EXISTS check_interactive_no_branching;

-- Drop threads indexes
DROP INDEX IF EXISTS idx_threads_created_at;
DROP INDEX IF EXISTS idx_threads_type_status;
DROP INDEX IF EXISTS idx_threads_type;