-- Add Skeets table
CREATE TABLE
  Skeets (
    skeet_id UUID PRIMARY KEY,
    content TEXT NOT NULL,
    published_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL
  );

CREATE INDEX idx_skeets_published_at ON Skeets (published_at DESC);