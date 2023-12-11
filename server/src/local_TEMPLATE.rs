use miette::{IntoDiagnostic, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let db = db::setup_db_pool().await?;

    let table_names = sqlx::query!(
        r#"
        SELECT tablename FROM pg_catalog.pg_tables
        WHERE 1=1
        AND schemaname != 'pg_catalog'
        AND schemaname != 'information_schema';
        "#,
    )
    .map(|row| row.tablename.unwrap())
    .fetch_all(&db)
    .await
    .into_diagnostic()?;

    println!("{:#?}", table_names);

    Ok(())
}
