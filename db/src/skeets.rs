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
    
    // Function to seed test data
    pub async fn seed_test_data(pool: &PgPool) -> Result<(), sqlx::Error> {
        // Create a few published skeets with different dates
        let mut skeet1 = Self::new("Just launched the POSSE system for my blog! Now my posts will automatically syndicate to different platforms.".to_string());
        skeet1.publish();
        
        let mut skeet2 = Self::new("Working on a new Rust project this weekend. Excited to share more soon!".to_string());
        skeet2.publish();
        
        let mut skeet3 = Self::new("TIL about the new Rust 1.77 features. The let-else improvements are particularly nice.".to_string());
        skeet3.publish();
        
        // Also create an unpublished skeet (this won't show up on the public page)
        let skeet4 = Self::new("This is a draft skeet that isn't published yet.".to_string());
        
        // Insert all skeets
        skeet1.insert(pool).await?;
        skeet2.insert(pool).await?;
        skeet3.insert(pool).await?;
        skeet4.insert(pool).await?;
        
        Ok(())
    }
}