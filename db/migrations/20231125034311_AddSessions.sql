-- Add migration script here
CREATE TABLE
  Sessions (
    session_id UUID PRIMARY KEY NOT NULL,
    user_id UUID REFERENCES Users (user_id) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
  );
