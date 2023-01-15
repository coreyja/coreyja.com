CREATE TABLE
  UserGithubLinks (
    id INTEGER PRIMARY KEY NOT NULL,
    user_id INTEGER REFERENCES Users (id) NOT NULL,
    external_github_username TEXT NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    access_token_expires_at DATETIME NOT NULL,
    refresh_token_expires_at DATETIME NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL
  );

CREATE UNIQUE INDEX uniq_UserGithubLinks_user_id ON UserGithubLinks (user_id);

CREATE UNIQUE INDEX uniq_UserGithubLinks_external_github_username ON UserGithubLinks (external_github_username);

CREATE TABLE
  UserGithubLinkStates (
    id INTEGER PRIMARY KEY NOT NULL,
    user_id INTEGER REFERENCES Users (id) NOT NULL,
    status VARCHAR(255) NOT NULL DEFAULT 'pending',
    state VARCHAR(255) NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL
  );
