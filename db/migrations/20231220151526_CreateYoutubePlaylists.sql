-- Add migration script here
CREATE TABLE
  YoutubePlaylists (
    youtube_playlist_id UUID PRIMARY KEY,
    external_youtube_playlist_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL
  );
  CREATE UNIQUE INDEX idx_youtube_playlists_external_youtube_playlist_id ON YoutubePlaylists (external_youtube_playlist_id);

CREATE TABLE
  YoutubeVideoPlaylists (
    youtube_video_playlist_id UUID PRIMARY KEY,
    youtube_playlist_id UUID NOT NULL REFERENCES YoutubePlaylists (youtube_playlist_id),
    youtube_video_id UUID NOT NULL REFERENCES YoutubeVideos (youtube_video_id),
    UNIQUE (youtube_playlist_id, youtube_video_id)
  );
