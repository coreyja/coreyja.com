//! Test scaffold for podcast episode parsing (BLOG-9266fbc5d7ec4044)
//!
//! These tests verify the PodcastEpisode and PodcastEpisodes types
//! that will be added to the posts crate as `posts::podcast`.
//!
//! All tests use `#[ignore]` because the `posts::podcast` module does not
//! exist yet. The implementation agent should:
//! 1. Create `posts/src/podcast.rs` with PodcastEpisodes and PodcastFrontMatter
//! 2. Add `pub mod podcast;` to `posts/src/lib.rs`
//! 3. Un-ignore these tests and uncomment the bodies

#[test]
#[ignore = "Requires posts::podcast module to be implemented"]
fn test_parse_podcast_episodes_from_static_dir() {
    // let episodes = posts::podcast::PodcastEpisodes::from_static_dir().unwrap();
    // assert!(
    //     !episodes.episodes.is_empty(),
    //     "Should parse at least one podcast episode from the static directory"
    // );
}

#[test]
#[ignore = "Requires posts::podcast module to be implemented"]
fn test_episode_frontmatter_fields() {
    // let episodes = posts::podcast::PodcastEpisodes::from_static_dir().unwrap();
    // let ep = &episodes.episodes[0];
    // assert!(!ep.frontmatter.title.is_empty(), "Episode title should not be empty");
    // assert!(!ep.frontmatter.slug.is_empty(), "Episode slug should not be empty");
    // assert!(!ep.frontmatter.youtube_id.is_empty(), "Episode youtube_id should not be empty");
    // assert!(!ep.frontmatter.audio_url.is_empty(), "Episode audio_url should not be empty");
    // assert!(ep.frontmatter.audio_length_bytes > 0, "Episode audio_length_bytes should be positive");
    // assert!(!ep.frontmatter.audio_duration.is_empty(), "Episode audio_duration should not be empty");
}

#[test]
#[ignore = "Requires posts::podcast module to be implemented"]
fn test_hello_world_episode_parses_correctly() {
    // let episodes = posts::podcast::PodcastEpisodes::from_static_dir().unwrap();
    // let ep = episodes.episodes.iter()
    //     .find(|e| e.frontmatter.slug == "hello-world")
    //     .expect("Should find the hello-world episode");
    // assert_eq!(ep.frontmatter.title, "Hello World - coreyja.fm Episode 1");
    // assert_eq!(ep.frontmatter.slug, "hello-world");
    // assert_eq!(ep.frontmatter.youtube_id, "dQw4w9WgXcQ");
    // assert_eq!(ep.frontmatter.audio_url, "https://coreyja-podcast.s3.amazonaws.com/episodes/hello-world.mp3");
    // assert_eq!(ep.frontmatter.audio_length_bytes, 12345678);
    // assert_eq!(ep.frontmatter.audio_duration, "00:45:30");
}

#[test]
#[ignore = "Requires posts::podcast module to be implemented"]
fn test_episodes_by_recency_returns_newest_first() {
    // let episodes = posts::podcast::PodcastEpisodes::from_static_dir().unwrap();
    // let sorted = episodes.by_recency();
    // assert!(!sorted.is_empty(), "by_recency should return at least one episode");
    // for window in sorted.windows(2) {
    //     use posts::date::PostedOn;
    //     assert!(
    //         window[0].posted_on() >= window[1].posted_on(),
    //         "Episodes should be sorted newest first"
    //     );
    // }
}

#[test]
#[ignore = "Requires posts::podcast module to be implemented"]
fn test_episode_implements_posted_on() {
    // use posts::date::PostedOn;
    // let episodes = posts::podcast::PodcastEpisodes::from_static_dir().unwrap();
    // let ep = &episodes.episodes[0];
    // let date = ep.posted_on();
    // assert_eq!(date, chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap());
}

#[test]
#[ignore = "Requires posts::podcast module to be implemented"]
fn test_episode_implements_title() {
    // use posts::title::Title;
    // let episodes = posts::podcast::PodcastEpisodes::from_static_dir().unwrap();
    // let ep = &episodes.episodes[0];
    // assert_eq!(ep.title(), "Hello World - coreyja.fm Episode 1");
}

#[test]
#[ignore = "Requires posts::podcast module to be implemented"]
fn test_episode_has_markdown_body() {
    // let episodes = posts::podcast::PodcastEpisodes::from_static_dir().unwrap();
    // let ep = episodes.episodes.iter()
    //     .find(|e| e.frontmatter.slug == "hello-world").unwrap();
    // assert!(!ep.ast.0.children.is_empty(), "Episode AST should have children");
}
