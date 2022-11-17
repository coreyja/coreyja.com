-- Add migration script here
ALTER TABLE
  DiscordTwitchLinks
ADD COLUMN
  twitch_user_id TEXT NOT NULL;

CREATE UNIQUE INDEX DiscordTwitchLinks_discord_user_id_uindex ON DiscordTwitchLinks (discord_user_id);

CREATE UNIQUE INDEX DiscordTwitchLinks_twitch_user_id_uindex ON DiscordTwitchLinks (twitch_user_id);
