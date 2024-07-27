use sqlx::PgPool;

pub async fn preferred_twitch_name(
    pool: &PgPool,
    twitch_username: &str,
) -> color_eyre::Result<String> {
    let user = get_or_create_twitch_chatter(pool, twitch_username).await?;

    Ok(user
        .preferred_name
        .unwrap_or_else(|| twitch_username.to_string()))
}

pub struct TwitchChatter {
    pub twitch_username: String,
    pub preferred_name: Option<String>,
}

pub async fn get_or_create_twitch_chatter(
    pool: &PgPool,
    twitch_username: &str,
) -> color_eyre::Result<TwitchChatter> {
    let user = sqlx::query_as!(
        TwitchChatter,
        r#"
        SELECT *
        FROM TwitchChatters
        WHERE twitch_username = $1
        "#,
        twitch_username
    )
    .fetch_optional(pool)
    .await?;

    if let Some(user) = user {
        Ok(user)
    } else {
        let user = sqlx::query_as!(
            TwitchChatter,
            r#"
            INSERT INTO TwitchChatters (twitch_username)
            VALUES ($1)
            RETURNING *
            "#,
            twitch_username
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }
}

pub async fn update_twitch_chatter_nickname(
    db: &PgPool,
    user: &TwitchChatter,
    new_nickname: &str,
) -> color_eyre::Result<()> {
    sqlx::query_as!(
        TwitchChatter,
        r#"
    UPDATE TwitchChatters
    SET preferred_name = $1
    WHERE twitch_username = $2
    "#,
        new_nickname,
        user.twitch_username
    )
    .execute(db)
    .await?;

    Ok(())
}
