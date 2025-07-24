-- Drop webhook_url column from discord_thread_metadata table
ALTER TABLE discord_thread_metadata
DROP COLUMN IF EXISTS webhook_url;