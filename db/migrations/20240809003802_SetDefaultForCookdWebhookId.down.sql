-- Add migration script here
ALTER TABLE CookdWebhooks
ALTER COLUMN cookd_webhook_id
DROP DEFAULT;
