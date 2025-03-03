use regex::Regex;
use rsky_lexicon::app::bsky::feed::{GetPostThreadOutput, ThreadViewPost, ThreadViewPostEnum};
use url::Url;

pub mod firehose;

use cja::Result;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use serde_json::Value;
use tracing::info;

// Simplified Jetstream model structures for testing
#[derive(Debug, Serialize)]
struct OptionsUpdate {
    #[serde(rename = "type")]
    message_type: String,
    payload: OptionsUpdatePayload,
}

#[derive(Debug, Serialize)]
struct OptionsUpdatePayload {
    wantedCollections: Vec<String>,
    wantedDids: Vec<String>,
    maxMessageSizeBytes: i32,
}

pub async fn test_connect() -> Result<()> {
    let handle = "coreyja.com";
    info!("Starting direct API test for handle: {}", handle);
    
    // Resolve the DID
    let did = firehose::get_did_for_handle(handle).await?;
    info!("Resolved DID: {}", did);
    
    // Initialize a database connection
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = sqlx::PgPool::connect(&database_url).await?;
    
    // Clear the Skeets table
    info!("Clearing skeets table...");
    sqlx::query!("TRUNCATE TABLE skeets").execute(&pool).await?;
    
    // Fetch the author's feed using Bluesky API
    let url = format!("https://bsky.social/xrpc/app.bsky.feed.getAuthorFeed?actor={}", handle);
    info!("Fetching author feed from: {}", url);
    
    let response = reqwest::get(&url).await?;
    info!("Got response with status: {}", response.status());
    
    let feed_data = response.json::<Value>().await?;
    
    // Extract and process posts
    let mut imported_count = 0;
    
    if let Some(feed) = feed_data.get("feed").and_then(|f| f.as_array()) {
        info!("Found {} posts in feed", feed.len());
        
        for post in feed {
            // Get the post data
            if let Some(post_view) = post.get("post") {
                if let Some(record) = post_view.get("record") {
                    // Extract text
                    let text = record.get("text").and_then(|t| t.as_str()).unwrap_or("");
                    info!("Post text: {}", text);
                    
                    // Check if it's a reply
                    if record.get("reply").is_some() {
                        info!("This is a reply - skipping");
                        continue;
                    }
                    
                    // Get post URI and create URL
                    if let Some(uri) = post_view.get("uri").and_then(|u| u.as_str()) {
                        // Extract the post ID from the URI
                        if let Some(post_id) = uri.split('/').last() {
                            let bsky_url = format!("https://bsky.app/profile/{}/post/{}", handle, post_id);
                            info!("Bluesky URL: {}", bsky_url);
                            
                            // Get creation date
                            let created_at_str = record.get("createdAt").and_then(|d| d.as_str());
                            let created_at = if let Some(date_str) = created_at_str {
                                match chrono::DateTime::parse_from_rfc3339(date_str) {
                                    Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
                                    Err(e) => {
                                        info!("Error parsing date: {}", e);
                                        None
                                    }
                                }
                            } else {
                                None
                            };
                            
                            // Create skeet in database
                            let skeet = db::skeets::Skeet::from_bluesky(
                                text.to_string(),
                                bsky_url,
                                created_at
                            );
                            
                            if let Err(e) = skeet.insert(&pool).await {
                                info!("Error inserting skeet: {}", e);
                            } else {
                                imported_count += 1;
                                info!("Imported skeet #{}", imported_count);
                            }
                        }
                    }
                }
            }
        }
    }
    
    info!("Successfully imported {} skeets", imported_count);
    
    // Check the skeets table
    let count = sqlx::query_scalar!("SELECT COUNT(*) FROM skeets")
        .fetch_one(&pool)
        .await?;
    
    info!("Found {} skeets in the database", count.unwrap_or(0));
    
    Ok(())
}

pub async fn fetch_thread(post_url: &str) -> cja::Result<ThreadViewPost> {
    let re = Regex::new(r"/profile/([\w.:]+)/post/([\w]+)").unwrap();
    let caps = re.captures(post_url).unwrap();

    let did = caps.get(1).unwrap().as_str();
    let post_id = caps.get(2).unwrap().as_str();

    let at_proto_uri = format!("at://{did}/app.bsky.feed.post/{post_id}");
    let mut url = Url::parse("https://public.api.bsky.app/xrpc/app.bsky.feed.getPostThread")?;
    url.set_query(Some(&format!("uri={at_proto_uri}")));

    let res = reqwest::get(url).await?;
    let data = res.json::<GetPostThreadOutput>().await?;

    let ThreadViewPostEnum::ThreadViewPost(thread) = data.thread else {
        return Err(cja::color_eyre::eyre::eyre!("Expected thread view post"));
    };

    Ok(thread)
}
