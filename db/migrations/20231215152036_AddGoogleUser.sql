-- Add migration script here
CREATE TABLE
  GoogleUsers (
    google_user_id UUID PRIMARY KEY NOT NULL,
    user_id UUID REFERENCES Users (user_id) NOT NULL,
    external_google_id TEXT NOT NULL,
    external_google_email TEXT NOT NULL,
    encrypted_access_token BYTEA NOT NULL,
    access_token_expires_at TIMESTAMPTZ NOT NULL,
    encrypted_refresh_token BYTEA NOT NULL,
    scope TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
  );

CREATE UNIQUE INDEX idx_google_users_external_google_id ON GoogleUsers (external_google_id);

CREATE UNIQUE INDEX idx_google_users_user_id ON GoogleUsers (user_id);
