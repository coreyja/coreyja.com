use sqlx::SqlitePool;

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

#[derive(Debug)]
pub(crate) struct DiscordGithubLink {
    pub(crate) id: i64,
    pub(crate) discord_user_id: i64,
    pub(crate) twitch_user_id: String,
    pub(crate) twitch_login: String,
}
