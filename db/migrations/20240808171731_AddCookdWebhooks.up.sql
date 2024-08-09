-- Add migration script here
CREATE TABLE
  CookdWebhooks (
    cookd_webhook_id UUID PRIMARY KEY,
    subdomain TEXT NOT NULL,
    slug TEXT NOT NULL,
    player_github_email TEXT NULL,
    player_github_username TEXT NULL,
    score INT NOT NULL,
    created_at TIMESTAMP
    WITH
      TIME ZONE NOT NULL,
      updated_at TIMESTAMP
    WITH
      TIME ZONE NOT NULL
  );
