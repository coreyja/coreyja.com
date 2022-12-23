-- Add migration script here
CREATE TABLE
  TwitchLinkStates (
    id INTEGER PRIMARY KEY NOT NULL,
    discord_user_id INTEGER NOT NULL,
    state VARCHAR(255) NOT NULL
  );

CREATE UNIQUE INDEX TwitchLinkStates_state_uindex ON TwitchLinkStates (state);

CREATE TABLE
  DiscordTwitchLinks (
    id INTEGER PRIMARY KEY NOT NULL,
    discord_user_id INTEGER NOT NULL,
    twitch_login TEXT NOT NULL
  );
