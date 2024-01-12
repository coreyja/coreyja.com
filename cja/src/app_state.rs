use crate::server::cookies::CookieKey;

pub trait AppState: Clone + Send + Sync + 'static {
    fn version(&self) -> &str;

    fn db(&self) -> &sqlx::PgPool;

    fn cookie_key(&self) -> &CookieKey;
}
