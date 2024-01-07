pub trait AppState: Clone + Send + Sync {
    fn version(&self) -> &str;

    fn db(&self) -> sqlx::PgPool;
}
