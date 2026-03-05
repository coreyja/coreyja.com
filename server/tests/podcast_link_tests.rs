//! Test scaffold for PodcastEpisode LinkTo implementation (BLOG-9266fbc5d7ec4044)
//!
//! Verifies that PodcastEpisode generates correct relative and absolute links.

#[test]
#[ignore = "Requires posts::podcast and server LinkTo impl for PodcastEpisode"]
fn test_podcast_episode_relative_link() {
    // PodcastEpisode with slug "hello-world" should produce "/podcast/hello-world"
    //
    // use server::http_server::LinkTo;
    // let episodes = posts::podcast::PodcastEpisodes::from_static_dir().unwrap();
    // let ep = episodes.episodes.iter()
    //     .find(|e| e.frontmatter.slug == "hello-world").unwrap();
    // assert_eq!(ep.relative_link(), "/podcast/hello-world");
    todo!("Implement after LinkTo for PodcastEpisode exists")
}

#[test]
#[ignore = "Requires posts::podcast and server LinkTo impl for PodcastEpisode"]
fn test_podcast_episode_absolute_link() {
    // With base_url "https://coreyja.com", absolute link should be
    // "https://coreyja.com/podcast/hello-world"
    todo!("Implement after LinkTo for PodcastEpisode exists")
}
