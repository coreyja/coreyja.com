-- Add migration script here
CREATE TABLE
  GithubSponsors (
    github_sponsor_id UUID PRIMARY KEY NOT NULL,
    user_id UUID REFERENCES Users (user_id) UNIQUE,
    sponsor_type TEXT NOT NULL,
    github_id TEXT NOT NULL,
    github_login TEXT NOT NULL,
    sponsored_at TIMESTAMPTZ NOT NULL,
    is_active BOOLEAN NOT NULL,
    is_one_time_payment BOOLEAN NOT NULL,
    privacy_level TEXT NOT NULL,
    amount_cents INT,
    tier_name TEXT
  );
