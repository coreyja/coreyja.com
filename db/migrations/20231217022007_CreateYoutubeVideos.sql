-- Add migration script here
-- youtube_video_id
-- external_youtube_id,
-- title,
-- description,
-- published_at,
-- thumbnail_url,
-- channel_id,
-- channel_title
CREATE TABLE
  YoutubeVideos (
    youtube_video_id UUID PRIMARY KEY,
    external_youtube_id text NOT NULL,
    title text NOT NULL,
    description TEXT,
    published_at TIMESTAMP,
    thumbnail_url text
  );

CREATE UNIQUE INDEX idx_youtube_videos_external_youtube_id ON YoutubeVideos (external_youtube_id);
