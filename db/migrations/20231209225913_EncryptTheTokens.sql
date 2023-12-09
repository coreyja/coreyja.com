-- Add migration script here
ALTER TABLE GithubLinks
DROP COLUMN access_token,
DROP COLUMN refresh_token,
ADD COLUMN encrypted_access_token bytea NOT NULL,
ADD COLUMN encrypted_refresh_token bytea NOT NULL;
