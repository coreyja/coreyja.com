-- Steps from: https://www.sqlite.org/lang_altertable.html#making_other_kinds_of_table_schema_changes
;

-- 1. If foreign key constraints are enabled, disable them using PRAGMA foreign_keys=OFF.
PRAGMA foreign_keys = off;

-- -- 2. Start a transaction.
-- BEGIN TRANSACTION;
-- 3. Remember the format of all indexes, triggers, and views associated with table X. This information will be needed in step 8 below. One way to do this is to run a query like the following: SELECT type, sql FROM sqlite_schema WHERE tbl_name='X'.
-- Here we just lookup the indexes and add them manually to this file
;

-- 4. Use CREATE TABLE to construct a new table "new_X" that is in the desired revised format of table X
CREATE TABLE
  DiscordTwitchLinks_new (
    id INTEGER PRIMARY KEY NOT NULL,
    discord_user_id INTEGER NOT NULL,
    twitch_login TEXT NOT NULL,
    twitch_user_id TEXT NOT NULL
  );

-- 5. Transfer Data
INSERT INTO
  DiscordTwitchLinks_new
SELECT
  *
FROM
  DiscordTwitchLinks;

-- 6. Drop the old table X
DROP TABLE DiscordTwitchLinks;

-- 7. Rename the new table "new_X" to "X"
ALTER TABLE DiscordTwitchLinks_new
RENAME TO DiscordTwitchLinks;

-- 8. Re-create all indexes, triggers, and views associated with table X. This information was saved in step 3 above.
CREATE UNIQUE INDEX DiscordTwitchLinks_discord_user_id_uindex ON DiscordTwitchLinks (discord_user_id);

CREATE UNIQUE INDEX DiscordTwitchLinks_twitch_user_id_uindex ON DiscordTwitchLinks (twitch_user_id);

-- 9. If any views refer to table X in a way that is affected by the schema change, then drop those views using DROP VIEW and recreate them with whatever changes are necessary to accommodate the schema change using CREATE VIEW.
-- Not Applicable Here
;

-- 10. If foreign key constraints were originally enabled then run PRAGMA foreign_key_check to verify that the schema change did not break any foreign key constraints.
PRAGMA foreign_key_check;

-- -- 11. Commit the transaction started in step 2.
-- COMMIT;
-- 12. If foreign key constraints were originally enabled, then re-enable them
PRAGMA foreign_keys = on;
