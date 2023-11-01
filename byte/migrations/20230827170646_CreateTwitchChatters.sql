-- Add migration script here
CREATE TABLE
  TwitchChatters (
    twitch_username TEXT NOT NULL,
    preferred_name TEXT NOT NULL
  );

CREATE INDEX TwitchChatters_twitch_username ON TwitchChatters (twitch_username);
