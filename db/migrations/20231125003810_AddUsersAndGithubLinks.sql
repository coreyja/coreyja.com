-- Add migration script here
CREATE TABLE
  Users (
    user_id UUID PRIMARY KEY NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
  );

CREATE TABLE
  GithubLinks (
    github_link_id UUID PRIMARY KEY NOT NULL,
    user_id UUID REFERENCES Users (user_id) NOT NULL,
    external_github_login TEXT NOT NULL,
    external_github_id TEXT NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    access_token_expires_at TIMESTAMPTZ NOT NULL,
    refresh_token_expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
  );

CREATE UNIQUE INDEX idx_github_links_user_id on GithubLinks (user_id);
