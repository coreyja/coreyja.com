use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

pub async fn create_test_db() -> PgPool {
    let db_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/coreyja_test".to_string());
    
    // Create a unique database for this test run
    let test_db_name = format!("test_{}_{}", std::process::id(), Uuid::new_v4());
    let base_url = db_url.rsplit_once('/').unwrap().0;
    let test_db_url = format!("{}/{}", base_url, test_db_name);
    
    // Connect to postgres to create the test database
    let maintenance_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await
        .expect("Failed to connect to postgres");
    
    sqlx::query(&format!("CREATE DATABASE \"{}\"", test_db_name))
        .execute(&maintenance_pool)
        .await
        .expect("Failed to create test database");
    
    // Connect to the new test database
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&test_db_url)
        .await
        .expect("Failed to connect to test database");
    
    // Run migrations
    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    pool
}