use crate::*;

use sqlx::SqlitePool;

mod models;
pub use models::*;

mod findable;
pub use findable::*;

#[derive(Debug)]
pub(crate) struct DiscordTwitchLink {
    pub(crate) id: i64,
    pub(crate) discord_user_id: i64,
    pub(crate) twitch_user_id: String,
    pub(crate) twitch_login: String,
}

pub(crate) async fn discord_twitch_link_from_user_id(
    discord_user_id: i64,
    db_pool: &SqlitePool,
) -> Result<Option<DiscordTwitchLink>, sqlx::Error> {
    sqlx::query_as!(
        DiscordTwitchLink,
        "SELECT * FROM DiscordTwitchLinks WHERE discord_user_id = ?",
        discord_user_id
    )
    .fetch_optional(db_pool)
    .await
}

pub(crate) async fn user_from_discord_user_id(
    discord_user_id: i64,
    db_pool: &SqlitePool,
) -> Result<QueryOnRead<User>, sqlx::Error> {
    let existing_user_id: Option<i64> = sqlx::query!(
        "SELECT user_id FROM UserDiscordLinks WHERE external_discord_user_id = ?",
        discord_user_id,
    )
    .fetch_optional(db_pool)
    .await?
    .map(|x| x.user_id);

    if let Some(existing_user_id) = existing_user_id {
        Ok(existing_user_id.into())
    } else {
        let user: User = sqlx::query_as!(User, "INSERT INTO Users DEFAULT VALUES RETURNING *",)
            .fetch_one(db_pool)
            .await?;

        sqlx::query!(
            "INSERT INTO UserDiscordLinks (user_id, external_discord_user_id) VALUES (?, ?)",
            user.id,
            discord_user_id
        )
        .fetch_optional(db_pool)
        .await?;

        Ok(user.into())
    }
}

pub async fn user_twitch_link_from_user(
    user: &QueryOnRead<User>,
    db_pool: &SqlitePool,
) -> Result<Option<UserTwitchLink>, sqlx::Error> {
    let user_id = user.id();
    sqlx::query_as!(
        UserTwitchLink,
        "SELECT * FROM UserTwitchLinks WHERE user_id = ?",
        user_id
    )
    .fetch_optional(db_pool)
    .await
}
