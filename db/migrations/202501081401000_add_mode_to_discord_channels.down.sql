-- Remove mode column and constraint from DiscordChannels table
ALTER TABLE DiscordChannels DROP CONSTRAINT IF EXISTS discord_channels_mode_check;
ALTER TABLE DiscordChannels DROP COLUMN IF EXISTS mode;