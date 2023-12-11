-- Add migration script here
CREATE TABLE
  GithubLoginStates (
    github_login_state_id UUID PRIMARY KEY,
    github_link_id UUID REFERENCES GithubLinks (github_link_id),
    app TEXT NOT NULL,
    state TEXT NOT NULL,
    created_at TIMESTAMP
    WITH
      TIME ZONE NOT NULL DEFAULT NOW ()
  );
