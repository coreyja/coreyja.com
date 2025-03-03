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

pub async fn start_bluesky_firehose(app_state: AppState) -> cja::Result<()> {
    let coreyja_handle = "coreyja.bsky.social"; // Your Bluesky handle
    let my_did = get_did_for_handle(coreyja_handle).await?;
    
    // Use one of the official Jetstream instances
    let jetstream_url = "wss://jetstream2.us-west.bsky.network/subscribe";
    
    info!("Starting Bluesky Jetstream connection for DID: {}", my_did);
    
    // Start the WebSocket connection loop
    loop {
        match connect_to_jetstream(jetstream_url, &app_state, &my_did, coreyja_handle).await {
            Ok(_) => {
                info!("Bluesky Jetstream connection closed, reconnecting in 5 seconds...");
            }
            Err(e) => {
                error!("Error in Bluesky Jetstream connection: {}", e);
                info!("Reconnecting in 5 seconds...");
            }
        }
        
        // Wait before reconnecting
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

async fn connect_to_jetstream(
    base_url: &str, 
    app_state: &AppState, 
    my_did: &str,
    handle: &str,
) -> cja::Result<()> {
    // Build the URL with query parameters to filter for our user
    let mut url = Url::parse(base_url)?;
    
    // We only want post records from our user
    url.query_pairs_mut()
        .append_pair("wantedCollections", "app.bsky.feed.post")
        .append_pair("wantedDids", my_did);
    
    // Connect to the Jetstream WebSocket
    info!("Connecting to Jetstream at: {}", url);
    let (ws_stream, _) = connect_async(url).await?;
    
    info!("Connected to Bluesky Jetstream");
    
    // Process messages
    process_jetstream(ws_stream, app_state, my_did, handle).await
}

async fn process_jetstream(
    mut ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    app_state: &AppState,
    my_did: &str,
    handle: &str,
) -> cja::Result<()> {
    // Process incoming messages
    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Err(e) = handle_jetstream_message(text, &app_state.db, my_did, handle).await {
                    error!("Error processing Jetstream message: {}", e);
                }
            }
            Ok(Message::Ping(data)) => {
                // Respond to ping with pong
                if let Err(e) = ws_stream.send(Message::Pong(data)).await {
                    error!("Error sending pong: {}", e);
                }
            }
            Ok(Message::Close(_)) => {
                info!("Jetstream WebSocket connection closed");
                break;
            }
            Err(e) => {
                error!("Jetstream WebSocket error: {}", e);
                break;
            }
            _ => {} // Ignore other message types
        }
    }
    
    Ok(())
}

async fn handle_jetstream_message(
    message: String, 
    pool: &PgPool, 
    my_did: &str,
    handle: &str,
) -> cja::Result<()> {
    // Parse the Jetstream event
    let event: JetstreamEvent = match serde_json::from_str(&message) {
        Ok(event) => event,
        Err(e) => {
            error!("Failed to parse Jetstream event: {}\nMessage: {}", e, message);
            return Ok(());
        }
    };
    
    // We're only interested in commit events from our user
    if event.kind != "commit" || event.did != my_did {
        return Ok(());
    }
    
    let Some(commit) = event.commit else {
        return Ok(());
    };
    
    // We only want create operations for posts
    if commit.operation.operation != "create" || commit.operation.collection != "app.bsky.feed.post" {
        return Ok(());
    }
    
    // The post record should be present for create operations
    let Some(record) = &commit.operation.record else {
        return Ok(());
    };
    
    // Extract the post content
    let content = match record.get("text") {
        Some(Value::String(text)) => text.clone(),
        _ => {
            warn!("Post record doesn't have text field: {:?}", record);
            return Ok(());
        }
    };
    
    // Skip empty posts or posts that just contain a URL
    if content.trim().is_empty() {
        return Ok(());
    }
    
    // Create the Bluesky URL for the post
    let post_id = commit.operation.rkey;
    let bsky_url = format!("https://bsky.app/profile/{}/post/{}", handle, post_id);
    
    // Check if we already have this post
    let existing = sqlx::query!(
        "SELECT skeet_id FROM Skeets WHERE bsky_url = $1",
        bsky_url
    )
    .fetch_optional(pool)
    .await?;
    
    if existing.is_some() {
        // Skip if already imported
        return Ok(());
    }
    
    // Create and save the skeet
    let skeet = Skeet::from_bluesky(content, bsky_url);
    skeet.insert(pool).await?;
    
    info!("Imported new post from Bluesky: {}", post_id);
    
    Ok(())
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