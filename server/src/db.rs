use sqlx::SqlitePool;

#[derive(Debug)]
pub struct DiscordTwitchLink {
    id: i64,
    discord_user_id: i64,
    twitch_user_id: String,
    twitch_login: String,
}

pub async fn discord_twitch_link_from_user_id(
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
