use chrono::Utc;
use db::skeets::Skeet;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};
use tracing::{error, info, warn};
use url::Url;

use crate::state::AppState;

// Simplified Jetstream model structures
#[derive(Debug, Deserialize)]
struct CommitOperation {
    operation: String,
    collection: String,
    rkey: String,
    record: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct CommitEvent {
    rev: String,
    #[serde(flatten)]
    operation: CommitOperation,
}

#[derive(Debug, Deserialize)]
struct JetstreamEvent {
    did: String,
    time_us: u64,
    kind: String,
    #[serde(default)]
    commit: Option<CommitEvent>,
}

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

// Get the most recent cursor value from the database, or return 0 to start from beginning
async fn get_stored_cursor(pool: &sqlx::PgPool) -> cja::Result<i64> {
    // Check if we have any cursor stored
    let result = sqlx::query!(
        "SELECT cursor_value FROM BlueskyJetstreamCursor ORDER BY updated_at DESC LIMIT 1"
    )
    .fetch_optional(pool)
    .await?;
    
    match result {
        Some(record) => Ok(record.cursor_value),
        None => {
            // No cursor found, start from beginning (0)
            info!("No previous cursor found, starting from beginning of Bluesky history");
            Ok(0)
        }
    }
}

// Store the cursor value to the database
async fn store_cursor(pool: &sqlx::PgPool, cursor_value: i64) -> cja::Result<()> {
    sqlx::query!(
        "INSERT INTO BlueskyJetstreamCursor (cursor_value, updated_at) VALUES ($1, $2)",
        cursor_value,
        chrono::Utc::now()
    )
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn start_bluesky_firehose(app_state: AppState) -> cja::Result<()> {
    let coreyja_handle = "coreyja.com"; // Your Bluesky handle
    let my_did = get_did_for_handle(coreyja_handle).await?;
    
    // Use one of the official Jetstream instances
    let jetstream_url = "wss://jetstream2.us-west.bsky.network/subscribe";
    
    info!("Starting Bluesky Jetstream connection for DID: {}", my_did);
    
    // Get initial cursor from database
    let mut cursor = get_stored_cursor(&app_state.db).await?;
    info!("Starting with cursor: {}", cursor);
    
    // Start the WebSocket connection loop
    loop {
        match connect_to_jetstream(jetstream_url, &app_state, &my_did, coreyja_handle, cursor).await {
            Ok(new_cursor) => {
                info!("Bluesky Jetstream connection closed, last cursor: {}", new_cursor);
                // Update our cursor for the next connection
                cursor = new_cursor;
                // Store cursor in database for persistence across restarts
                if let Err(e) = store_cursor(&app_state.db, cursor).await {
                    error!("Failed to store cursor: {}", e);
                }
            }
            Err(e) => {
                error!("Error in Bluesky Jetstream connection: {}", e);
            }
        }
        
        info!("Reconnecting in 5 seconds...");
        // Wait before reconnecting
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

pub async fn connect_to_jetstream(
    base_url: &str, 
    app_state: &AppState, 
    my_did: &str,
    handle: &str,
    cursor: i64,
) -> cja::Result<i64> {
    // Build the URL with query parameters to filter for our user
    let mut url = Url::parse(base_url)?;
    
    // We only want post records from our user
    url.query_pairs_mut()
        .append_pair("wantedCollections", "app.bsky.feed.post")
        .append_pair("wantedDids", my_did);
    
    // Add cursor if it's not zero (zero is default for starting from beginning)
    if cursor > 0 {
        url.query_pairs_mut().append_pair("cursor", &cursor.to_string());
    }
    
    // Connect to the Jetstream WebSocket
    info!("Connecting to Jetstream at: {}", url);
    let (mut ws_stream, _) = connect_async(url).await?;
    
    info!("Connected to Bluesky Jetstream");
    
    // Send a subscribe message
    let options = OptionsUpdate {
        message_type: "subscribe".to_string(),
        payload: OptionsUpdatePayload {
            wantedCollections: vec!["app.bsky.feed.post".to_string()],
            wantedDids: vec![my_did.to_string()],
            maxMessageSizeBytes: 1024 * 1024, // 1MB
        },
    };
    
    let subscribe_msg = serde_json::to_string(&options)?;
    info!("Sending subscribe message: {}", subscribe_msg);
    ws_stream.send(Message::Text(subscribe_msg)).await?;
    
    // Process messages
    process_jetstream(ws_stream, app_state, my_did, handle).await
}

async fn process_jetstream(
    mut ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    app_state: &AppState,
    my_did: &str,
    handle: &str,
) -> cja::Result<i64> {
    // Keep track of the most recent cursor
    let mut latest_cursor: i64 = 0;
    let mut message_count = 0;
    
    info!("Starting to process Jetstream WebSocket messages");
    
    // Process incoming messages
    while let Some(msg) = ws_stream.next().await {
        message_count += 1;
        if message_count % 10 == 0 {
            info!("Processed {} Jetstream messages", message_count);
        }
        
        match msg {
            Ok(Message::Text(text)) => {
                info!("Received text message of length: {}", text.len());
                if text.len() < 200 {
                    info!("Short message content: {}", text);
                }
                
                match handle_jetstream_message(text, &app_state.db, my_did, handle).await {
                    Ok(cursor) => {
                        // Update our latest cursor if this event has a newer one
                        if cursor > latest_cursor {
                            latest_cursor = cursor;
                            
                            // Store cursor periodically (every 100 events)
                            if latest_cursor % 100 == 0 {
                                if let Err(e) = store_cursor(&app_state.db, latest_cursor).await {
                                    error!("Failed to store cursor: {}", e);
                                } else {
                                    info!("Stored cursor: {}", latest_cursor);
                                }
                            }
                        }
                    },
                    Err(e) => {
                        error!("Error handling Jetstream message: {}", e);
                    }
                }
            }
            Ok(Message::Ping(data)) => {
                info!("Received WebSocket ping");
                // Respond to ping with pong
                if let Err(e) = ws_stream.send(Message::Pong(data)).await {
                    error!("Error sending pong: {}", e);
                } else {
                    info!("Sent pong response");
                }
            }
            Ok(Message::Close(frame)) => {
                if let Some(frame) = frame {
                    info!("Jetstream WebSocket connection closed with code {}: {}", 
                          frame.code, frame.reason);
                } else {
                    info!("Jetstream WebSocket connection closed without frame");
                }
                break;
            }
            Ok(other) => {
                info!("Received other WebSocket message type: {:?}", other);
            }
            Err(e) => {
                error!("Jetstream WebSocket error: {}", e);
                break;
            }
        }
    }
    
    // Return the latest cursor we've seen
    Ok(latest_cursor)
}

async fn handle_jetstream_message(
    message: String, 
    pool: &PgPool, 
    my_did: &str,
    handle: &str,
) -> cja::Result<i64> {
    // Parse the Jetstream event
    let event: JetstreamEvent = match serde_json::from_str(&message) {
        Ok(event) => event,
        Err(e) => {
            error!("Failed to parse Jetstream event: {}\nMessage: {}", e, message);
            return Ok(0);
        }
    };
    
    // Always return the cursor timestamp regardless of whether we process the message
    let cursor = event.time_us as i64;
    
    // Log event information
    info!("Received Jetstream event: kind={}, did={}, cursor={}", event.kind, event.did, cursor);
    
    // We're only interested in commit events from our user
    if event.kind != "commit" || event.did != my_did {
        info!("Skipping event - not a commit event from our user: kind={}, did={}, our_did={}", 
              event.kind, event.did, my_did);
        return Ok(cursor);
    }
    
    let Some(commit) = event.commit else {
        info!("Skipping event - no commit data");
        return Ok(cursor);
    };
    
    // We only want create operations for posts
    if commit.operation.operation != "create" || commit.operation.collection != "app.bsky.feed.post" {
        info!("Skipping event - not a create operation for app.bsky.feed.post: operation={}, collection={}", 
              commit.operation.operation, commit.operation.collection);
        return Ok(cursor);
    }
    
    info!("Found create operation for post: rkey={}", commit.operation.rkey);

    // The post record should be present for create operations
    let Some(record) = &commit.operation.record else {
        info!("Skipping event - no record data in the operation");
        return Ok(cursor);
    };
    
    // Check if this is a reply - we want to skip replies
    if record.get("reply").is_some() {
        info!("Skipping reply post");
        return Ok(cursor);
    }
    
    // Extract the post content
    let content = match record.get("text") {
        Some(Value::String(text)) => text.clone(),
        _ => {
            warn!("Post record doesn't have text field: {:?}", record);
            return Ok(cursor);
        }
    };
    
    info!("Found post content: {}", content);
    
    // Skip empty posts or posts that just contain a URL
    if content.trim().is_empty() {
        info!("Skipping empty post");
        return Ok(cursor);
    }
    
    // Extract original creation date
    let created_at = match record.get("createdAt") {
        Some(Value::String(created_at_str)) => {
            match chrono::DateTime::parse_from_rfc3339(created_at_str) {
                Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
                Err(e) => {
                    warn!("Failed to parse createdAt date: {}", e);
                    None
                }
            }
        },
        _ => None,
    };
    
    // Create the Bluesky URL for the post
    let post_id = commit.operation.rkey;
    let bsky_url = format!("https://bsky.app/profile/{}/post/{}", handle, post_id);
    
    info!("Generated Bluesky URL: {}", bsky_url);
    
    // Check if we already have this post
    let existing = sqlx::query!(
        "SELECT skeet_id FROM Skeets WHERE bsky_url = $1",
        bsky_url
    )
    .fetch_optional(pool)
    .await?;
    
    if existing.is_some() {
        info!("Skipping post - already imported: {}", post_id);
        return Ok(cursor);
    }
    
    // Create and save the skeet with the original publication date
    let skeet = Skeet::from_bluesky(content, bsky_url, created_at);
    match skeet.insert(pool).await {
        Ok(_) => info!("Successfully imported new post from Bluesky: {}", post_id),
        Err(e) => error!("Failed to import skeet: {}", e),
    };
    
    Ok(cursor)
}

pub async fn get_did_for_handle(handle: &str) -> cja::Result<String> {
    let url = format!("https://bsky.social/xrpc/com.atproto.identity.resolveHandle?handle={}", handle);
    
    tracing::info!("Resolving Bluesky handle: {}", handle);
    tracing::info!("Resolve URL: {}", url);
    
    let response_raw = reqwest::get(&url).await?;
    tracing::info!("Got response with status: {}", response_raw.status());
    
    let response = response_raw.json::<serde_json::Value>().await?;
    tracing::info!("Response: {:?}", response);
    
    let did = response["did"].as_str()
        .ok_or_else(|| cja::color_eyre::eyre::eyre!("Failed to get DID from response"))?
        .to_string();
    
    tracing::info!("Resolved DID: {}", did);
    
    Ok(did)
}