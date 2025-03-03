use cja::Result;
use db::skeets::Skeet;
use tracing::info;

pub(crate) async fn seed_skeets() -> Result<()> {
    info!("Seeding Skeets table with test data");
    
    let db_pool = db::setup_db_pool().await?;
    Skeet::seed_test_data(&db_pool).await?;
    
    info!("Successfully seeded Skeets table");
    Ok(())
}