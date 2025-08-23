-- Add back author_user_id column to recipes table
ALTER TABLE recipes ADD COLUMN author_user_id UUID NOT NULL;