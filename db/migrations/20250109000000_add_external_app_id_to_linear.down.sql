-- Remove external_app_id column from linear_installations
ALTER TABLE linear_installations
DROP COLUMN external_app_id;