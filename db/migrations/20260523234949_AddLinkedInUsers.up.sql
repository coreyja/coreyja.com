CREATE TABLE LinkedInUsers (
  linkedin_user_id UUID PRIMARY KEY NOT NULL,
  user_id UUID REFERENCES Users (user_id) NOT NULL,
  external_linkedin_id TEXT NOT NULL,
  encrypted_access_token BYTEA NOT NULL,
  access_token_expires_at TIMESTAMPTZ NOT NULL,
  encrypted_refresh_token BYTEA NOT NULL,
  refresh_token_expires_at TIMESTAMPTZ NOT NULL,
  scope TEXT NOT NULL,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE UNIQUE INDEX idx_linkedin_users_external_linkedin_id ON LinkedInUsers (external_linkedin_id);
CREATE UNIQUE INDEX idx_linkedin_users_user_id ON LinkedInUsers (user_id);

-- OAuth CSRF state tracking. Each `/admin/auth/linkedin` redirect inserts a
-- row; the callback validates the `state` query param against this table and
-- rejects states older than 10 minutes. DB-backed state instead of signed
-- cookies matches the existing LinearOauthStates pattern in this codebase and
-- avoids adding new dependencies.
CREATE TABLE LinkedInOauthStates (
  linkedin_oauth_state_id UUID PRIMARY KEY NOT NULL,
  state TEXT NOT NULL,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- Index for the callback's freshness check and any future cleanup job that
-- needs to sweep stale rows (mirrors idx_linear_oauth_states_created_at).
CREATE INDEX idx_linkedin_oauth_states_created_at ON LinkedInOauthStates (created_at);
