//! Tests for podcast episode parsing (BLOG-9266fbc5d7ec4044)

#[test]
fn test_parse_podcast_episodes_from_static_dir() {
    let episodes = posts::podcast::PodcastEpisodes::from_static_dir().unwrap();
    assert!(
        !episodes.episodes.is_empty(),
        "Should parse at least one podcast episode from the static directory"
    );
}

#[test]
fn test_episode_frontmatter_fields() {
    let episodes = posts::podcast::PodcastEpisodes::from_static_dir().unwrap();
    let ep = &episodes.episodes[0];
    assert!(
        !ep.frontmatter.title.is_empty(),
        "Episode title should not be empty"
    );
    assert!(
        !ep.frontmatter.slug.is_empty(),
        "Episode slug should not be empty"
    );
    assert!(
        !ep.frontmatter.youtube_id.is_empty(),
        "Episode youtube_id should not be empty"
    );
    assert!(
        !ep.frontmatter.audio_url.is_empty(),
        "Episode audio_url should not be empty"
    );
    assert!(
        ep.frontmatter.audio_length_bytes > 0,
        "Episode audio_length_bytes should be positive"
    );
    assert!(
        !ep.frontmatter.audio_duration.is_empty(),
        "Episode audio_duration should not be empty"
    );
}

#[test]
fn test_hello_world_episode_parses_correctly() {
    let episodes = posts::podcast::PodcastEpisodes::from_static_dir().unwrap();
    let ep = episodes
        .episodes
        .iter()
        .find(|e| e.frontmatter.slug == "why-im-starting-a-podcast")
        .expect("Should find the hello-world episode");
    assert_eq!(
        ep.frontmatter.title,
        "Why I'm Starting a Podcast (and What I've Been Building)"
    );
    assert_eq!(ep.frontmatter.slug, "why-im-starting-a-podcast");
    assert_eq!(
        ep.frontmatter.audio_url,
        "https://coreyja-fm.s3.us-east-2.amazonaws.com/001/audio.mp3"
    );
    assert_eq!(ep.frontmatter.audio_length_bytes, 10_777_931);
    assert_eq!(ep.frontmatter.audio_duration, "00:22:27");
    assert_eq!(ep.frontmatter.youtube_id, "B12ZggV47rg");
    assert_eq!(
        ep.frontmatter.youtube_url.as_deref(),
        Some("https://youtu.be/B12ZggV47rg")
    );
    assert_eq!(
        ep.frontmatter.transcript_url.as_deref(),
        Some("https://coreyja-fm.s3.us-east-2.amazonaws.com/001/transcript.srt")
    );
}

#[test]
fn test_episodes_by_recency_returns_newest_first() {
    let episodes = posts::podcast::PodcastEpisodes::from_static_dir().unwrap();
    let sorted = episodes.by_recency();
    assert!(
        !sorted.is_empty(),
        "by_recency should return at least one episode"
    );
    for window in sorted.windows(2) {
        use posts::date::PostedOn;
        assert!(
            window[0].posted_on() >= window[1].posted_on(),
            "Episodes should be sorted newest first"
        );
    }
}

#[test]
fn test_episode_implements_posted_on() {
    use posts::date::PostedOn;
    let episodes = posts::podcast::PodcastEpisodes::from_static_dir().unwrap();
    let ep = &episodes.episodes[0];
    let date = ep.posted_on();
    assert_eq!(date, chrono::NaiveDate::from_ymd_opt(2026, 3, 6).unwrap());
}

#[test]
fn test_episode_implements_title() {
    use posts::title::Title;
    let episodes = posts::podcast::PodcastEpisodes::from_static_dir().unwrap();
    let ep = &episodes.episodes[0];
    assert_eq!(
        ep.title(),
        "Why I'm Starting a Podcast (and What I've Been Building)"
    );
}

#[test]
fn test_episode_has_markdown_body() {
    let episodes = posts::podcast::PodcastEpisodes::from_static_dir().unwrap();
    let ep = episodes
        .episodes
        .iter()
        .find(|e| e.frontmatter.slug == "why-im-starting-a-podcast")
        .unwrap();
    assert!(
        !ep.ast.0.children.is_empty(),
        "Episode AST should have children"
    );
}
