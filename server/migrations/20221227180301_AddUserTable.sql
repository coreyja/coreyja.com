-- Add migration script here
CREATE TABLE
  Users (
    id INTEGER PRIMARY KEY NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL
  );

CREATE TABLE
  UserDiscordLinks (
    id INTEGER PRIMARY KEY NOT NULL,
    user_id INTEGER REFERENCES Users (id) NOT NULL,
    external_discord_user_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL
  );

CREATE UNIQUE INDEX uniq_UserDiscordLinks_user_id ON UserDiscordLinks (user_id);

CREATE UNIQUE INDEX uniq_UserDiscordLinks_external_discord_user_id ON UserDiscordLinks (external_discord_user_id);

CREATE TABLE
  UserTwitchLinks (
    id INTEGER PRIMARY KEY NOT NULL,
    user_id INTEGER REFERENCES Users (id) NOT NULL,
    external_twitch_login TEXT NOT NULL,
    external_twitch_user_id INTEGER NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    access_token_expires_at DATETIME NOT NULL,
    access_token_validated_at DATETIME NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL
  );

CREATE UNIQUE INDEX uniq_UserTwitchLinks_user_id ON UserTwitchLinks (user_id);

CREATE UNIQUE INDEX uniq_UserTwitchLinks_external_twitch_user_id ON UserTwitchLinks (external_twitch_user_id);

CREATE TABLE
  UserTwitchLinkStates (
    id INTEGER PRIMARY KEY NOT NULL,
    user_id INTEGER REFERENCES Users (id) NOT NULL,
    status VARCHAR(255) NOT NULL DEFAULT 'pending',
    state VARCHAR(255) NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL
  );
