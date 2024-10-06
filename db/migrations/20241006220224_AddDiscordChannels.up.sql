-- Add migration script here
CREATE TABLE
  DiscordChannels (
    discord_channel_id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    channel_name TEXT NULL,
    channel_topic TEXT NULL,
    channel_id TEXT NOT NULL,
    purpose TEXT NOT NULL,
    created_at TIMESTAMP
    WITH
      TIME ZONE NOT NULL DEFAULT now (),
      updated_at TIMESTAMP
    WITH
      TIME ZONE NOT NULL DEFAULT now ()
  );

CREATE INDEX ON DiscordChannels (purpose);
