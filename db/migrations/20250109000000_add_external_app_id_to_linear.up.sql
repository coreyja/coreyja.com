-- Add external_app_id column to linear_installations
ALTER TABLE linear_installations
ADD COLUMN external_app_id text;