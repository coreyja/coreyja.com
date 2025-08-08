-- Remove mode column and constraint from threads table
ALTER TABLE threads DROP CONSTRAINT IF EXISTS threads_mode_check;
ALTER TABLE threads DROP COLUMN IF EXISTS mode;