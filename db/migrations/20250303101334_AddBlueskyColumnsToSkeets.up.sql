-- Add Bluesky-related columns to the Skeets table
ALTER TABLE Skeets
  ADD COLUMN imported_from_bluesky_at TIMESTAMP WITH TIME ZONE,
  ADD COLUMN published_on_bsky_at TIMESTAMP WITH TIME ZONE,
  ADD COLUMN bsky_url TEXT;