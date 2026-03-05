//! Test scaffold for podcast HTTP routes (BLOG-9266fbc5d7ec4044)
//!
//! These tests verify the podcast page handlers:
//! - GET /podcast (index listing all episodes)
//! - GET /podcast/{slug} (individual episode with YouTube embed)
//! - GET /podcast/feed.xml (RSS feed with enclosures)
//!
//! All tests require the podcast module and routes to be implemented,
//! as well as PodcastEpisodes to be added to AppState.

// NOTE: These tests require a running database (PgPool) due to the test_helpers
// creating an AppState with a DB connection. If the implementation adds a way to
// create a test router without DB, these can be simplified.
//
// Since these tests need both the podcast module AND the test_helpers to be updated
// to include podcast_episodes in AppState, they are all #[ignore]d.

#[tokio::test]
#[ignore = "Requires podcast routes and AppState.podcast_episodes to be implemented"]
async fn test_podcast_index_returns_200() {
    // Once implemented, this test should:
    // 1. Create a test app with create_test_app()
    // 2. Send GET /podcast
    // 3. Assert 200 status
    // 4. Assert response body contains "coreyja.fm" or "Podcast"
    //
    // Example:
    // let app = server::http_server::test_helpers::create_test_app(pool).await;
    // let response = app.oneshot(
    //     axum::http::Request::builder()
    //         .uri("/podcast")
    //         .body(axum::body::Body::empty())
    //         .unwrap()
    // ).await.unwrap();
    // assert_eq!(response.status(), 200);
    todo!("Implement after podcast routes exist")
}

#[tokio::test]
#[ignore = "Requires podcast routes and AppState.podcast_episodes to be implemented"]
async fn test_podcast_index_contains_episode_links() {
    // Should verify the index page lists episodes with links
    // Assert response body contains href="/podcast/hello-world"
    todo!("Implement after podcast routes exist")
}

#[tokio::test]
#[ignore = "Requires podcast routes and AppState.podcast_episodes to be implemented"]
async fn test_podcast_episode_returns_200() {
    // GET /podcast/hello-world should return 200
    todo!("Implement after podcast routes exist")
}

#[tokio::test]
#[ignore = "Requires podcast routes and AppState.podcast_episodes to be implemented"]
async fn test_podcast_episode_contains_youtube_embed() {
    // GET /podcast/hello-world should contain a YouTube iframe
    // Assert body contains "youtube.com/embed/dQw4w9WgXcQ"
    todo!("Implement after podcast routes exist")
}

#[tokio::test]
#[ignore = "Requires podcast routes and AppState.podcast_episodes to be implemented"]
async fn test_podcast_episode_not_found_returns_404() {
    // GET /podcast/nonexistent-slug should return 404
    todo!("Implement after podcast routes exist")
}

#[tokio::test]
#[ignore = "Requires podcast routes and AppState.podcast_episodes to be implemented"]
async fn test_podcast_rss_feed_returns_xml_content_type() {
    // GET /podcast/feed.xml should return Content-Type: application/rss+xml
    todo!("Implement after podcast routes exist")
}

#[tokio::test]
#[ignore = "Requires podcast routes and AppState.podcast_episodes to be implemented"]
async fn test_podcast_rss_feed_contains_channel_metadata() {
    // RSS feed should contain:
    // - <title>coreyja.fm</title>
    // - <description> with podcast description
    // - <link> pointing to /podcast
    todo!("Implement after podcast routes exist")
}

#[tokio::test]
#[ignore = "Requires podcast routes and AppState.podcast_episodes to be implemented"]
async fn test_podcast_rss_feed_contains_enclosure() {
    // Each RSS item should have an <enclosure> element with:
    // - url pointing to the audio file
    // - type="audio/mpeg"
    // - length matching audio_length_bytes
    todo!("Implement after podcast routes exist")
}

#[tokio::test]
#[ignore = "Requires podcast routes and AppState.podcast_episodes to be implemented"]
async fn test_podcast_rss_feed_contains_itunes_extensions() {
    // RSS feed should include iTunes namespace extensions:
    // - <itunes:duration> on items
    // - <itunes:author> on channel/items
    todo!("Implement after podcast routes exist")
}

#[tokio::test]
#[ignore = "Requires podcast routes and AppState.podcast_episodes to be implemented"]
async fn test_podcast_rss_feed_items_ordered_by_recency() {
    // RSS feed items should be ordered newest first
    // (same ordering as the index page)
    todo!("Implement after podcast routes exist")
}
