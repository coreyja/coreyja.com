-- Add migration script here
ALTER TABLE Sessions
ALTER COLUMN session_id
SET DEFAULT gen_random_uuid ();
