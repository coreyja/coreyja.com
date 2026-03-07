use std::path::PathBuf;

use chrono::NaiveDate;
use include_dir::{include_dir, Dir, File};
use markdown::mdast::Node;
use serde::{Deserialize, Serialize};

use crate::{MarkdownAst, Post};

use super::{
    blog::ValidateMarkdown,
    date::{ByRecency, PostedOn},
    title::Title,
};

use color_eyre::{eyre::Context, Result};

pub(crate) static NOTES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../notes");

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NoteKind {
    Til,
    Link,
}

#[derive(Debug, Clone)]
pub struct NotePosts {
    pub posts: Vec<NotePost>,
}

pub type NotePost = Post<FrontMatter>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct FrontMatter {
    pub title: String,
    pub date: NaiveDate,
    pub slug: String,
    pub kind: Option<NoteKind>,
    pub bsky_url: Option<String>,
}

impl PostedOn for FrontMatter {
    fn posted_on(&self) -> chrono::NaiveDate {
        self.date
    }
}

impl Title for FrontMatter {
    fn title(&self) -> &str {
        &self.title
    }
}

impl NotePost {
    fn from_file(file: &File) -> Result<Self> {
        let ast = MarkdownAst::from_file(file)?;
        let metadata: FrontMatter = ast.frontmatter()?;
        let path = file.path().to_owned();

        Ok(Self {
            ast,
            path,
            frontmatter: metadata,
        })
    }

    pub(crate) fn validate(&self) -> Result<()> {
        self.validate_images()?;

        Ok(())
    }

    fn validate_images(&self) -> Result<()> {
        let p = &self.frontmatter.slug;
        let p = PathBuf::from(p);

        let root_node = Node::Root(self.ast.0.clone());
        root_node.validate_images(&p)?;

        Ok(())
    }
}

impl NotePosts {
    pub fn from_static_dir() -> Result<Self> {
        Self::from_dir(&NOTES_DIR)
    }

    pub fn from_dir(dir: &Dir) -> Result<Self> {
        let posts = dir
            .find("**/*.md")?
            .filter_map(|e| e.as_file())
            .map(NotePost::from_file)
            .collect::<Result<Vec<_>>>()
            .wrap_err("One of the notes failed to parse")?;

        Ok(Self { posts })
    }

    pub fn validate(&self) -> Result<()> {
        println!("Validating Slug Uniqueness");
        for slug in self.posts.iter().map(|note| &note.frontmatter.slug) {
            let matches: Vec<_> = self
                .posts
                .iter()
                .filter(|note| &note.frontmatter.slug == slug)
                .collect();
            if matches.len() > 1 {
                let paths = matches
                    .iter()
                    .map(|note| note.path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(color_eyre::eyre::eyre!(
                    "Slug {} is not unique. Found these paths {}",
                    slug,
                    paths
                ));
            }
        }

        println!("Validating {} notes", self.posts.len());
        for note in &self.posts {
            println!(
                "Validating {} from {}...",
                note.frontmatter.slug,
                note.path.display()
            );

            note.validate()?;
        }
        println!("Notes Valid! ✅");

        Ok(())
    }

    pub fn by_recency(&self) -> Vec<&NotePost> {
        self.posts.by_recency()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmatter_deserializes_with_til_kind() {
        let yaml = r"
title: Test Note
date: 2026-03-01
slug: test-note
kind: til
";
        let fm: FrontMatter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(fm.title, "Test Note");
        assert_eq!(fm.slug, "test-note");
        assert_eq!(fm.kind, Some(NoteKind::Til));
        assert_eq!(fm.bsky_url, None);
    }

    #[test]
    fn frontmatter_deserializes_with_link_kind() {
        let yaml = r"
title: Cool Link
date: 2026-03-02
slug: cool-link
kind: link
";
        let fm: FrontMatter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(fm.kind, Some(NoteKind::Link));
    }

    #[test]
    fn frontmatter_deserializes_without_kind() {
        let yaml = r"
title: Just a Note
date: 2026-03-03
slug: just-a-note
";
        let fm: FrontMatter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(fm.title, "Just a Note");
        assert_eq!(fm.kind, None);
    }

    #[test]
    fn frontmatter_deserializes_with_bsky_url() {
        let yaml = r"
title: Syndicated Note
date: 2026-03-04
slug: syndicated-note
kind: til
bsky_url: https://bsky.app/profile/coreyja.com/post/abc123
";
        let fm: FrontMatter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            fm.bsky_url,
            Some("https://bsky.app/profile/coreyja.com/post/abc123".to_string())
        );
    }

    #[test]
    fn frontmatter_deserializes_without_bsky_url() {
        let yaml = r"
title: Unsyndicated Note
date: 2026-03-05
slug: unsyndicated
kind: til
";
        let fm: FrontMatter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(fm.bsky_url, None);
    }

    #[test]
    fn frontmatter_rejects_invalid_kind() {
        let yaml = r"
title: Bad Kind
date: 2026-03-06
slug: bad-kind
kind: invalid_kind
";
        let result: Result<FrontMatter, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err(), "Invalid kind should fail deserialization");
    }

    #[test]
    fn frontmatter_roundtrips_through_serde() {
        let yaml = r"
title: Roundtrip Test
date: 2026-03-07
slug: roundtrip
kind: til
bsky_url: https://bsky.app/profile/coreyja.com/post/xyz
";
        let fm: FrontMatter = serde_yaml::from_str(yaml).unwrap();
        let serialized = serde_yaml::to_string(&fm).unwrap();
        let deserialized: FrontMatter = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(fm, deserialized);
    }

    #[test]
    fn notes_load_from_static_dir() {
        let notes = NotePosts::from_static_dir().unwrap();
        assert!(
            !notes.posts.is_empty(),
            "Should have at least one note loaded from the notes/ directory"
        );
    }

    #[test]
    fn notes_validate_slug_uniqueness() {
        let notes = NotePosts::from_static_dir().unwrap();
        assert!(notes.validate().is_ok());
    }

    #[test]
    fn notes_by_recency_returns_newest_first() {
        let notes = NotePosts::from_static_dir().unwrap();
        let sorted = notes.by_recency();
        if sorted.len() >= 2 {
            for window in sorted.windows(2) {
                assert!(
                    window[0].frontmatter.date >= window[1].frontmatter.date,
                    "Notes should be sorted by date descending"
                );
            }
        }
    }

    #[test]
    fn migrated_tils_have_kind_til() {
        let notes = NotePosts::from_static_dir().unwrap();
        let til_notes: Vec<_> = notes
            .posts
            .iter()
            .filter(|n| n.frontmatter.kind == Some(NoteKind::Til))
            .collect();
        assert!(
            !til_notes.is_empty(),
            "Should have at least one note with kind: til (migrated from TILs)"
        );
    }
}
