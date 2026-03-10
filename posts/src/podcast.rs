use chrono::NaiveDate;
use include_dir::{include_dir, Dir, File};
use serde::{Deserialize, Serialize};

use crate::{MarkdownAst, Post};

use super::{
    date::{ByRecency, PostedOn},
    title::Title,
};

use color_eyre::{eyre::Context, Result};

static PODCAST_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../podcast");

#[derive(Debug, Clone)]
pub struct PodcastEpisodes {
    pub episodes: Vec<PodcastEpisode>,
}

pub type PodcastEpisode = Post<PodcastFrontMatter>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct PodcastFrontMatter {
    pub title: String,
    pub date: NaiveDate,
    pub slug: String,
    pub youtube_id: String,
    pub youtube_url: Option<String>,
    pub audio_url: String,
    /// File size in bytes for the RSS enclosure
    pub audio_length_bytes: u64,
    /// Duration as HH:MM:SS string for iTunes metadata
    pub audio_duration: String,
    /// SRT transcript URL for podcast apps (Podcasting 2.0 transcript tag)
    pub transcript_url: Option<String>,
}

impl PostedOn for PodcastFrontMatter {
    fn posted_on(&self) -> NaiveDate {
        self.date
    }
}

impl Title for PodcastFrontMatter {
    fn title(&self) -> &str {
        &self.title
    }
}

impl PodcastEpisode {
    fn from_file(file: &File) -> Result<Self> {
        let ast = MarkdownAst::from_file(file)?;
        let metadata: PodcastFrontMatter = ast.frontmatter()?;
        let path = file.path().to_owned();
        Ok(Self {
            ast,
            path,
            frontmatter: metadata,
        })
    }
}

impl PodcastEpisodes {
    pub fn from_static_dir() -> Result<Self> {
        Self::from_dir(&PODCAST_DIR)
    }

    pub fn from_dir(dir: &Dir) -> Result<Self> {
        let episodes = dir
            .find("**/*.md")?
            .filter_map(|e| e.as_file())
            .map(PodcastEpisode::from_file)
            .collect::<Result<Vec<_>>>()
            .wrap_err("One of the podcast episodes failed to parse")?;
        Ok(Self { episodes })
    }

    pub fn by_recency(&self) -> Vec<&PodcastEpisode> {
        self.episodes.by_recency()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_podcast_episodes() {
        let episodes = PodcastEpisodes::from_static_dir().unwrap();
        assert!(!episodes.episodes.is_empty());
        let ep = &episodes.episodes[0];
        assert!(!ep.frontmatter.slug.is_empty());
        assert!(!ep.frontmatter.youtube_id.is_empty());
        assert!(!ep.frontmatter.audio_url.is_empty());
    }
}
