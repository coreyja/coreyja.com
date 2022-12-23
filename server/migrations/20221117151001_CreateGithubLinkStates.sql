-- Add migration script here
CREATE TABLE
  GithubLinkStates (
    id INTEGER PRIMARY KEY NOT NULL,
    discord_user_id INTEGER NOT NULL,
    state VARCHAR(255) NOT NULL
  );

CREATE UNIQUE INDEX GithubLinkStates_state_uindex ON GithubLinkStates (state);
