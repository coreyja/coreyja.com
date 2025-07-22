-- Add database-level constraints and indexes for better data integrity and performance

-- 1. Threads table constraints
-- Ensure interactive threads don't have a branching_stitch_id (they're always top-level)
ALTER TABLE threads ADD CONSTRAINT check_interactive_no_branching
    CHECK (thread_type != 'interactive' OR branching_stitch_id IS NULL);

-- Add index for thread_type to speed up filtering
CREATE INDEX idx_threads_type ON threads(thread_type);

-- Add composite index for common query pattern
CREATE INDEX idx_threads_type_status ON threads(thread_type, status);

-- Add index for created_at for time-based queries
CREATE INDEX idx_threads_created_at ON threads(created_at DESC);

-- 2. Stitches table constraints
-- Ensure stitch type matches required fields
ALTER TABLE stitches ADD CONSTRAINT check_discord_message_fields
    CHECK (stitch_type != 'discord_message' OR llm_request IS NOT NULL);

-- Ensure stitches can't reference themselves
ALTER TABLE stitches ADD CONSTRAINT check_no_self_reference
    CHECK (stitch_id != previous_stitch_id);

-- Add composite index for thread_id + created_at for ordered retrieval
CREATE INDEX idx_stitches_thread_created ON stitches(thread_id, created_at);

-- Add index on child_thread_id for finding parent stitches
CREATE INDEX idx_stitches_child_thread ON stitches(child_thread_id) WHERE child_thread_id IS NOT NULL;

-- 3. Discord metadata constraints
-- Ensure Discord IDs are valid snowflakes (numeric strings)
ALTER TABLE discord_thread_metadata ADD CONSTRAINT check_discord_thread_id_format
    CHECK (discord_thread_id ~ '^\d+$');

ALTER TABLE discord_thread_metadata ADD CONSTRAINT check_channel_id_format
    CHECK (channel_id ~ '^\d+$');

ALTER TABLE discord_thread_metadata ADD CONSTRAINT check_guild_id_format
    CHECK (guild_id ~ '^\d+$');

ALTER TABLE discord_thread_metadata ADD CONSTRAINT check_last_message_id_format
    CHECK (last_message_id IS NULL OR last_message_id ~ '^\d+$');

-- Ensure webhook URL is valid if provided
ALTER TABLE discord_thread_metadata ADD CONSTRAINT check_webhook_url_format
    CHECK (webhook_url IS NULL OR webhook_url ~ '^https://discord\.com/api/webhooks/\d+/.+$');

-- Ensure participants is an array
ALTER TABLE discord_thread_metadata ADD CONSTRAINT check_participants_is_array
    CHECK (jsonb_typeof(participants) = 'array');

-- Add composite index for guild + created_at for guild-specific queries
CREATE INDEX idx_discord_guild_created ON discord_thread_metadata(guild_id, created_at DESC);

