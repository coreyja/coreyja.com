-- Add migration script here
ALTER TABLE Sessions
ALTER COLUMN user_id
DROP NOT NULL;
