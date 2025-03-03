-- Remove the unique index on bsky_url
DROP INDEX idx_skeets_bsky_url_unique;

-- Drop cursor table
DROP TABLE BlueskyJetstreamCursor;