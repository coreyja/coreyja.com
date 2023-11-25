use miette::IntoDiagnostic;
use sqlx::{types::Uuid, PgPool};

pub struct User {
    pub user_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
