-- Rename external_app_id to external_actor_id
ALTER TABLE linear_installations
RENAME COLUMN external_app_id TO external_actor_id;