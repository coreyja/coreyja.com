-- Add migration script here
CREATE TABLE
  LastRefreshAt (
    key TEXT PRIMARY KEY NOT NULL,
    last_refresh_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
  );
