-- Add mode column to DiscordChannels table
ALTER TABLE DiscordChannels ADD COLUMN mode text;

-- Add check constraint for valid modes
ALTER TABLE DiscordChannels ADD CONSTRAINT discord_channels_mode_check 
    CHECK (mode IN ('general', 'cooking', 'project_manager'));

-- Update existing channels to have 'general' mode
UPDATE DiscordChannels SET mode = 'general' WHERE mode IS NULL;

-- Make mode NOT NULL with default
ALTER TABLE DiscordChannels ALTER COLUMN mode SET NOT NULL;
ALTER TABLE DiscordChannels ALTER COLUMN mode SET DEFAULT 'general';