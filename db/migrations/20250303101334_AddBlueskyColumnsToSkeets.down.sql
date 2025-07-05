-- Remove Bluesky-related columns from the Skeets table
ALTER TABLE Skeets
  DROP COLUMN imported_from_bluesky_at,
  DROP COLUMN published_on_bsky_at,
  DROP COLUMN bsky_url;