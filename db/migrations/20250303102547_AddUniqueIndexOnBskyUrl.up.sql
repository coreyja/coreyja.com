-- Add unique index on bsky_url for deduplication
CREATE UNIQUE INDEX idx_skeets_bsky_url_unique ON Skeets (bsky_url) WHERE bsky_url IS NOT NULL;

-- Create table to store Bluesky Jetstream cursor
CREATE TABLE BlueskyJetstreamCursor (
  id SERIAL PRIMARY KEY,
  cursor_value BIGINT NOT NULL,
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL
);