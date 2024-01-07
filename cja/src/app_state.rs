pub trait AppState: Clone + Send + Sync + 'static {
    fn version(&self) -> &str;

    fn db(&self) -> &sqlx::PgPool;
}
