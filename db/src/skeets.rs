use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skeet {
    pub skeet_id: Uuid,
    pub content: String,
    pub published_at: Option<DateTime<Utc>>,
    pub imported_from_bluesky_at: Option<DateTime<Utc>>,
    pub published_on_bsky_at: Option<DateTime<Utc>>,
    pub bsky_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Skeet {
    pub fn new(content: String) -> Self {
        let now = Utc::now();
        Self {
            skeet_id: Uuid::new_v4(),
            content,
            published_at: None, // Starts unpublished
            imported_from_bluesky_at: None,
            published_on_bsky_at: None,
            bsky_url: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn publish(&mut self) {
        self.published_at = Some(Utc::now());
        self.updated_at = Utc::now();
        
        // In the future, this is where we'll add logic to publish to various services
    }
    
    pub fn is_published(&self) -> bool {
        self.published_at.is_some()
    }
    
    pub fn from_bluesky(content: String, bsky_url: String) -> Self {
        let now = Utc::now();
        let mut skeet = Self::new(content);
        
        skeet.imported_from_bluesky_at = Some(now);
        skeet.bsky_url = Some(bsky_url);
        skeet.published_at = Some(now); // Auto-publish imported skeets
        
        skeet
    }
    
    pub async fn insert(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO Skeets (
                skeet_id, 
                content, 
                published_at, 
                imported_from_bluesky_at, 
                published_on_bsky_at,
                bsky_url,
                created_at, 
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (skeet_id) DO NOTHING
            "#,
            self.skeet_id,
            self.content,
            self.published_at,
            self.imported_from_bluesky_at,
            self.published_on_bsky_at,
            self.bsky_url,
            self.created_at,
            self.updated_at
        )
        .execute(pool)
        .await?;
        
        Ok(())
    }
    
    // Find a skeet by bsky_url
    pub async fn find_by_bsky_url(pool: &PgPool, bsky_url: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT * FROM Skeets WHERE bsky_url = $1
            "#,
            bsky_url
        )
        .fetch_optional(pool)
        .await
    }
    
    // Bulk insert Skeets efficiently
    pub async fn bulk_insert(skeets: &[Self], pool: &PgPool) -> Result<(), sqlx::Error> {
        // Start a transaction
        let mut tx = pool.begin().await?;
        
        for skeet in skeets {
            sqlx::query!(
                r#"
                INSERT INTO Skeets (
                    skeet_id, 
                    content, 
                    published_at, 
                    imported_from_bluesky_at, 
                    published_on_bsky_at,
                    bsky_url,
                    created_at, 
                    updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (bsky_url) DO NOTHING
                "#,
                skeet.skeet_id,
                skeet.content,
                skeet.published_at,
                skeet.imported_from_bluesky_at,
                skeet.published_on_bsky_at,
                skeet.bsky_url,
                skeet.created_at,
                skeet.updated_at
            )
            .execute(&mut *tx)
            .await?;
        }
        
        // Commit transaction
        tx.commit().await?;
        
        Ok(())
    }
}