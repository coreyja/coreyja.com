-- Add migration script here
ALTER TABLE CookdWebhooks
ALTER COLUMN cookd_webhook_id
SET DEFAULT gen_random_uuid ();
