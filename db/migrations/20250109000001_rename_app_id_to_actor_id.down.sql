-- Rename external_actor_id back to external_app_id
ALTER TABLE linear_installations
RENAME COLUMN external_actor_id TO external_app_id;