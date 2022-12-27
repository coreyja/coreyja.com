use crate::*;

use async_trait::async_trait;
use color_eyre::eyre::ContextCompat;
use sqlx::types::chrono::NaiveDateTime;

#[derive(Debug)]
pub struct User {
    pub id: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[async_trait]
impl Findable for User {
    async fn find(id: i64, pool: &mut sqlx::SqliteConnection) -> Result<User> {
        let user = sqlx::query_as!(User, "SELECT * FROM Users WHERE id = ?", id)
            .fetch_optional(pool)
            .await?;

        Ok(user.wrap_err("We are expecting the ids used with Findable to always exist")?)
    }

    fn id(&self) -> i64 {
        self.id
    }
}

pub struct UserTwitchLink {
    pub id: i64,
    pub user_id: i64,
    pub external_twitch_user_id: i64,
    pub external_twitch_login: String,
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_at: NaiveDateTime,
    pub access_token_validated_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub struct UserGithubLink {
    pub id: i64,
    pub user_id: i64,
    pub external_github_username: String,
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expires_at: NaiveDateTime,
    pub refresh_token_expires_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
