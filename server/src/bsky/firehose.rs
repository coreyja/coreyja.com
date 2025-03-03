use chrono::Utc;
use db::skeets::Skeet;
use regex::Regex;
use rsky_lexicon::app::bsky::feed::Post;
use sqlx::PgPool;
use tokio::time::{self, Duration};
use tracing::{info, error, warn};
use url::Url;

use crate::state::AppState;

pub async fn start_bluesky_firehose(app_state: AppState) -> cja::Result<()> {
    // Poll Bluesky API every 5 minutes for new posts
    let interval = Duration::from_secs(5 * 60);
    let mut interval = time::interval(interval);

    let coreyja_handle = "coreyja.bsky.social"; // Your Bluesky handle
    let did_regex = Regex::new(r"^did:plc:([a-z0-9]+)$").unwrap();
    
    info!("Starting Bluesky firehose for handle: {}", coreyja_handle);

    loop {
        interval.tick().await;
        
        match fetch_recent_posts(&app_state.db, coreyja_handle, &did_regex).await {
            Ok(num_posts) => {
                if num_posts > 0 {
                    info!("Imported {} posts from Bluesky", num_posts);
                }
            }
            Err(e) => {
                error!("Error fetching posts from Bluesky: {}", e);
            }
        }
    }
}

async fn fetch_recent_posts(pool: &PgPool, handle: &str, did_regex: &Regex) -> cja::Result<usize> {
    // Get author DID
    let author_did = get_did_for_handle(handle).await?;
    
    // Fetch recent posts for this author
    let posts = fetch_author_posts(&author_did).await?;
    
    if posts.is_empty() {
        return Ok(0);
    }
    
    let mut imported_count = 0;
    
    // Process posts
    for post in posts {
        // Skip replies and reposts for now, only import original posts
        if post.reply.is_some() {
            continue;
        }
        
        let content = post.text.clone();
        
        // Skip if this is just a URL without any text
        if content.trim().is_empty() {
            continue;
        }
        
        // Check if we already have this post by Bluesky URL
        let post_id = post.ref_to_uri(author_did.as_str()).split('/').last().unwrap_or_default();
        let bsky_url = format!("https://bsky.app/profile/{}/post/{}", handle, post_id);
        
        let existing = sqlx::query!(
            "SELECT skeet_id FROM Skeets WHERE bsky_url = $1",
            bsky_url
        )
        .fetch_optional(pool)
        .await?;
        
        if existing.is_some() {
            // Skip if we've already imported this post
            continue;
        }
        
        // Create and save the skeet
        let skeet = Skeet::from_bluesky(content, bsky_url);
        skeet.insert(pool).await?;
        
        imported_count += 1;
    }
    
    Ok(imported_count)
}

async fn get_did_for_handle(handle: &str) -> cja::Result<String> {
    let url = format!("https://bsky.social/xrpc/com.atproto.identity.resolveHandle?handle={}", handle);
    
    let response = reqwest::get(url).await?
        .json::<serde_json::Value>()
        .await?;
    
    let did = response["did"].as_str()
        .ok_or_else(|| cja::color_eyre::eyre::eyre!("Failed to get DID from response"))?
        .to_string();
    
    Ok(did)
}

async fn fetch_author_posts(author_did: &str) -> cja::Result<Vec<Post>> {
    let url = format!(
        "https://bsky.social/xrpc/app.bsky.feed.getAuthorFeed?actor={}&limit=20", 
        author_did
    );
    
    let response = reqwest::get(url).await?
        .json::<serde_json::Value>()
        .await?;
    
    let posts = response["feed"].as_array()
        .ok_or_else(|| cja::color_eyre::eyre::eyre!("Failed to get feed array"))?;
    
    let mut result = Vec::new();
    
    for feed_item in posts {
        if let Some(post) = feed_item["post"]["record"].as_object() {
            // Parse the raw post JSON into our Post struct
            if let Ok(parsed_post) = serde_json::from_value::<Post>(feed_item["post"]["record"].clone()) {
                result.push(parsed_post);
            }
        }
    }
    
    Ok(result)
}