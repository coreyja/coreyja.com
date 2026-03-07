// Test scaffold for notes module - BLOG-6a79edf75fad4d29
// The implementation agent should add the production code (NoteKind, FrontMatter, NotePost, NotePosts)
// above this test block and un-ignore the tests.

#[cfg(test)]
mod tests {
    // Tests will use: super::*
    // Required types: NoteKind, FrontMatter, NotePost, NotePosts

    #[test]
    #[ignore = "Requires NoteKind enum to be implemented"]
    fn frontmatter_deserializes_with_til_kind() {
        // FrontMatter with kind: til should deserialize successfully
        let yaml = r#"
title: Test Note
date: 2026-03-01
slug: test-note
kind: til
"#;
        let fm: super::FrontMatter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(fm.title, "Test Note");
        assert_eq!(fm.slug, "test-note");
        assert_eq!(fm.kind, Some(super::NoteKind::Til));
        assert_eq!(fm.bsky_url, None);
    }

    #[test]
    #[ignore = "Requires NoteKind enum to be implemented"]
    fn frontmatter_deserializes_with_link_kind() {
        let yaml = r#"
title: Cool Link
date: 2026-03-02
slug: cool-link
kind: link
"#;
        let fm: super::FrontMatter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(fm.kind, Some(super::NoteKind::Link));
    }

    #[test]
    #[ignore = "Requires FrontMatter with optional kind field"]
    fn frontmatter_deserializes_without_kind() {
        // kind is optional — notes without a kind are valid
        let yaml = r#"
title: Just a Note
date: 2026-03-03
slug: just-a-note
"#;
        let fm: super::FrontMatter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(fm.title, "Just a Note");
        assert_eq!(fm.kind, None);
    }

    #[test]
    #[ignore = "Requires FrontMatter with bsky_url field"]
    fn frontmatter_deserializes_with_bsky_url() {
        let yaml = r#"
title: Syndicated Note
date: 2026-03-04
slug: syndicated-note
kind: til
bsky_url: https://bsky.app/profile/coreyja.com/post/abc123
"#;
        let fm: super::FrontMatter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            fm.bsky_url,
            Some("https://bsky.app/profile/coreyja.com/post/abc123".to_string())
        );
    }

    #[test]
    #[ignore = "Requires FrontMatter with optional bsky_url field"]
    fn frontmatter_deserializes_without_bsky_url() {
        let yaml = r#"
title: Unsyndicated Note
date: 2026-03-05
slug: unsyndicated
kind: til
"#;
        let fm: super::FrontMatter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(fm.bsky_url, None);
    }

    #[test]
    #[ignore = "Requires NoteKind enum with snake_case serde"]
    fn frontmatter_rejects_invalid_kind() {
        // Invalid kind values should fail deserialization
        let yaml = r#"
title: Bad Kind
date: 2026-03-06
slug: bad-kind
kind: invalid_kind
"#;
        let result: Result<super::FrontMatter, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err(), "Invalid kind should fail deserialization");
    }

    #[test]
    #[ignore = "Requires FrontMatter to implement Serialize"]
    fn frontmatter_roundtrips_through_serde() {
        let yaml = r#"
title: Roundtrip Test
date: 2026-03-07
slug: roundtrip
kind: til
bsky_url: https://bsky.app/profile/coreyja.com/post/xyz
"#;
        let fm: super::FrontMatter = serde_yaml::from_str(yaml).unwrap();
        let serialized = serde_yaml::to_string(&fm).unwrap();
        let deserialized: super::FrontMatter = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(fm, deserialized);
    }

    #[test]
    #[ignore = "Requires NotePosts::from_static_dir to be implemented"]
    fn notes_load_from_static_dir() {
        // All notes in the notes/ directory should load successfully
        let notes = super::NotePosts::from_static_dir().unwrap();
        assert!(
            !notes.posts.is_empty(),
            "Should have at least one note loaded from the notes/ directory"
        );
    }

    #[test]
    #[ignore = "Requires NotePosts with slug uniqueness validation"]
    fn notes_validate_slug_uniqueness() {
        // Validation should catch duplicate slugs
        let notes = super::NotePosts::from_static_dir().unwrap();
        // If from_static_dir succeeds, validate should also succeed
        // (no duplicate slugs in the real content)
        assert!(notes.validate().is_ok());
    }

    #[test]
    #[ignore = "Requires NotePosts::by_recency"]
    fn notes_by_recency_returns_newest_first() {
        let notes = super::NotePosts::from_static_dir().unwrap();
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
    #[ignore = "Requires migrated notes with kind: til"]
    fn migrated_tils_have_kind_til() {
        // After migration, all former TIL files should have kind: til
        let notes = super::NotePosts::from_static_dir().unwrap();
        // There should be at least some notes with kind: til (the migrated TILs)
        let til_notes: Vec<_> = notes
            .posts
            .iter()
            .filter(|n| n.frontmatter.kind == Some(super::NoteKind::Til))
            .collect();
        assert!(
            !til_notes.is_empty(),
            "Should have at least one note with kind: til (migrated from TILs)"
        );
    }
}
