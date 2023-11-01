-- Add migration script here
ALTER TABLE TwitchChatters
ALTER COLUMN preferred_name
DROP NOT NULL;
