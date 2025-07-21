-- Create discord_thread_metadata table
CREATE TABLE discord_thread_metadata (
    thread_id UUID PRIMARY KEY REFERENCES threads(thread_id) ON DELETE CASCADE,
    discord_thread_id TEXT NOT NULL UNIQUE,
    channel_id TEXT NOT NULL,
    guild_id TEXT NOT NULL,
    last_message_id TEXT,
    created_by TEXT NOT NULL,
    thread_name TEXT NOT NULL,
    participants JSONB DEFAULT '[]'::jsonb NOT NULL,
    webhook_url TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- Create indexes for efficient lookups
CREATE INDEX idx_discord_thread_id ON discord_thread_metadata(discord_thread_id);
CREATE INDEX idx_discord_guild_id ON discord_thread_metadata(guild_id);
CREATE INDEX idx_discord_channel_id ON discord_thread_metadata(channel_id);

-- Create trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_discord_thread_metadata_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_discord_thread_metadata_updated_at
BEFORE UPDATE ON discord_thread_metadata
FOR EACH ROW
EXECUTE FUNCTION update_discord_thread_metadata_updated_at();

-- Remove the metadata column from threads table since we're moving Discord data to dedicated table
-- Note: This will lose any existing Discord metadata, but since this is Phase 1, there shouldn't be any production data yet
ALTER TABLE threads DROP COLUMN IF EXISTS metadata;

-- Remove the Discord-specific index
DROP INDEX IF EXISTS idx_threads_metadata_discord;