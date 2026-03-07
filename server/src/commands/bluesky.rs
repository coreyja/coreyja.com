// Test scaffold for Bluesky publish command - BLOG-6a79edf75fad4d29
// The implementation agent should add PublishBlueskyArgs, parse_frontmatter,
// update_frontmatter_with_bsky_url, strip_markdown, format_post_text, and publish_bluesky
// above this test block, then un-ignore tests.

#[cfg(test)]
mod tests {
    // Tests will use: super::*
    // Required functions: parse_frontmatter, update_frontmatter_with_bsky_url,
    //                     strip_markdown, format_post_text

    // ==================== parse_frontmatter tests ====================

    #[test]
    #[ignore = "Requires parse_frontmatter for note FrontMatter"]
    fn parse_frontmatter_basic_note() {
        let content = r#"---
title: Test Note
date: 2026-03-05
slug: test-note
kind: til
---

This is the body of the note.
"#;
        let (fm, body) = super::parse_frontmatter(content).unwrap();
        assert_eq!(fm.title, "Test Note");
        assert_eq!(fm.slug, "test-note");
        assert!(body.contains("This is the body of the note."));
    }

    #[test]
    #[ignore = "Requires parse_frontmatter for note FrontMatter"]
    fn parse_frontmatter_with_bsky_url() {
        let content = r#"---
title: Already Published
date: 2026-03-05
slug: already-published
kind: til
bsky_url: https://bsky.app/profile/coreyja.com/post/abc123
---

Body text.
"#;
        let (fm, _body) = super::parse_frontmatter(content).unwrap();
        assert_eq!(
            fm.bsky_url,
            Some("https://bsky.app/profile/coreyja.com/post/abc123".to_string())
        );
    }

    #[test]
    #[ignore = "Requires parse_frontmatter for note FrontMatter"]
    fn parse_frontmatter_without_kind() {
        let content = r#"---
title: Plain Note
date: 2026-03-06
slug: plain-note
---

Just a note without a kind.
"#;
        let (fm, _body) = super::parse_frontmatter(content).unwrap();
        assert_eq!(fm.kind, None);
    }

    #[test]
    #[ignore = "Requires parse_frontmatter"]
    fn parse_frontmatter_missing_opening_delimiter() {
        let content = r#"title: Bad
date: 2026-03-05
slug: bad
---

Body.
"#;
        assert!(
            super::parse_frontmatter(content).is_err(),
            "Should error on missing opening ---"
        );
    }

    #[test]
    #[ignore = "Requires parse_frontmatter"]
    fn parse_frontmatter_missing_closing_delimiter() {
        let content = r#"---
title: Bad
date: 2026-03-05
slug: bad

Body without closing delimiter.
"#;
        assert!(
            super::parse_frontmatter(content).is_err(),
            "Should error on missing closing ---"
        );
    }

    // ==================== update_frontmatter_with_bsky_url tests ====================

    #[test]
    #[ignore = "Requires update_frontmatter_with_bsky_url"]
    fn update_frontmatter_adds_bsky_url() {
        let content = r#"---
title: Test Note
date: 2026-03-05
slug: test-note
kind: til
---

Body content here.
"#;
        let updated = super::update_frontmatter_with_bsky_url(
            content,
            "https://bsky.app/profile/coreyja.com/post/abc123",
        );
        assert!(
            updated.contains("bsky_url: https://bsky.app/profile/coreyja.com/post/abc123"),
            "Should insert bsky_url into frontmatter"
        );
        assert!(
            updated.contains("Body content here."),
            "Should preserve body content"
        );
    }

    #[test]
    #[ignore = "Requires update_frontmatter_with_bsky_url"]
    fn update_frontmatter_preserves_all_existing_fields() {
        let content = r#"---
title: Full Note
date: 2026-03-05
slug: full-note
kind: link
---

Body.
"#;
        let updated = super::update_frontmatter_with_bsky_url(
            content,
            "https://bsky.app/profile/coreyja.com/post/xyz",
        );

        assert!(
            updated.contains("title: Full Note"),
            "Should preserve title"
        );
        assert!(updated.contains("date: 2026-03-05"), "Should preserve date");
        assert!(updated.contains("slug: full-note"), "Should preserve slug");
        assert!(updated.contains("kind: link"), "Should preserve kind");
        assert!(
            updated.contains("bsky_url: https://bsky.app/profile/coreyja.com/post/xyz"),
            "Should add bsky_url"
        );
    }

    #[test]
    #[ignore = "Requires update_frontmatter_with_bsky_url"]
    fn update_frontmatter_preserves_body_formatting() {
        let content = r#"---
title: Test
date: 2026-03-05
slug: test
---

# Heading

Paragraph with **bold** and _italic_.

- List item 1
- List item 2

```rust
fn main() {}
```
"#;
        let updated =
            super::update_frontmatter_with_bsky_url(content, "https://bsky.app/profile/x/post/y");

        assert!(updated.contains("# Heading"));
        assert!(updated.contains("**bold**"));
        assert!(updated.contains("- List item 1"));
        assert!(updated.contains("```rust"));
    }

    // ==================== strip_markdown tests ====================

    #[test]
    #[ignore = "Requires strip_markdown function"]
    fn strip_markdown_removes_bold() {
        let result = super::strip_markdown("This is **bold** text");
        assert_eq!(result, "This is bold text");
    }

    #[test]
    #[ignore = "Requires strip_markdown function"]
    fn strip_markdown_removes_italic() {
        let result = super::strip_markdown("This is _italic_ text");
        assert_eq!(result, "This is italic text");
    }

    #[test]
    #[ignore = "Requires strip_markdown function"]
    fn strip_markdown_removes_backticks() {
        let result = super::strip_markdown("Use `cargo build` to compile");
        assert_eq!(result, "Use cargo build to compile");
    }

    #[test]
    #[ignore = "Requires strip_markdown function"]
    fn strip_markdown_removes_heading_prefixes() {
        let result = super::strip_markdown("# Heading\n## Subheading");
        assert!(
            !result.contains('#'),
            "Should remove # heading prefixes, got: {result}"
        );
    }

    #[test]
    #[ignore = "Requires strip_markdown function"]
    fn strip_markdown_converts_links_to_text() {
        let result = super::strip_markdown("Check out [this site](https://example.com)");
        assert!(
            result.contains("this site"),
            "Should keep link text, got: {result}"
        );
        assert!(
            !result.contains('['),
            "Should remove bracket syntax, got: {result}"
        );
        assert!(
            !result.contains("https://example.com"),
            "Should remove link URL from text, got: {result}"
        );
    }

    #[test]
    #[ignore = "Requires strip_markdown function"]
    fn strip_markdown_plain_text_unchanged() {
        let text = "Just plain text with no formatting";
        let result = super::strip_markdown(text);
        assert_eq!(result, text);
    }

    // ==================== format_post_text tests ====================

    #[test]
    #[ignore = "Requires format_post_text function"]
    fn format_post_text_basic() {
        let result = super::format_post_text(
            "My Title",
            "This is the body of the note.",
            "https://coreyja.com/notes/my-note",
        );
        assert!(result.contains("My Title"), "Should contain title");
        assert!(result.contains("This is the body"), "Should contain body");
        assert!(
            result.contains("https://coreyja.com/notes/my-note"),
            "Should contain URL"
        );
    }

    #[test]
    #[ignore = "Requires format_post_text function"]
    fn format_post_text_structure() {
        // Format should be: "{title}\n\n{body}\n\n{url}"
        let result =
            super::format_post_text("Title", "Body text", "https://coreyja.com/notes/slug");
        let lines: Vec<&str> = result.split('\n').collect();
        assert_eq!(lines[0], "Title", "First line should be title");
        assert_eq!(lines[1], "", "Second line should be blank");
        // Body starts on line 3
        // Then blank line
        // Then URL
        assert!(
            result.ends_with("https://coreyja.com/notes/slug"),
            "Should end with URL"
        );
    }

    #[test]
    #[ignore = "Requires format_post_text function"]
    fn format_post_text_under_300_chars_no_truncation() {
        let title = "Short Title";
        let body = "Short body.";
        let url = "https://coreyja.com/notes/short";
        let result = super::format_post_text(title, body, url);

        assert!(
            result.chars().count() <= 300,
            "Short post should be under 300 chars"
        );
        assert!(
            result.contains("Short body."),
            "Short body should not be truncated"
        );
    }

    #[test]
    #[ignore = "Requires format_post_text function"]
    fn format_post_text_truncates_body_when_over_300_chars() {
        let title = "Title";
        let body = "A".repeat(400); // Way over 300 chars
        let url = "https://coreyja.com/notes/long-note";
        let result = super::format_post_text(title, &body, url);

        assert!(
            result.chars().count() <= 300,
            "Total post text must not exceed 300 characters, got {}",
            result.chars().count()
        );
        assert!(result.contains("Title"), "Title should never be truncated");
        assert!(result.contains(url), "URL should never be truncated");
    }

    #[test]
    #[ignore = "Requires format_post_text function"]
    fn format_post_text_preserves_title_and_url_when_truncating() {
        let title = "My Important Title";
        let body = "x".repeat(500);
        let url = "https://coreyja.com/notes/important";
        let result = super::format_post_text(title, &body, url);

        assert!(
            result.starts_with("My Important Title"),
            "Title must be intact"
        );
        assert!(result.ends_with(url), "URL must be intact at end");
        assert!(
            result.chars().count() <= 300,
            "Must be within 300 char limit"
        );
    }

    #[test]
    #[ignore = "Requires format_post_text function"]
    fn format_post_text_uses_char_count_not_byte_count() {
        // Bluesky's 300 char limit is character-based, not byte-based
        let title = "🦀 Rust";
        let body = "é".repeat(300); // Each é is 2 bytes but 1 char
        let url = "https://coreyja.com/notes/emoji";
        let result = super::format_post_text(title, &body, url);

        // Should truncate based on chars, not bytes
        assert!(
            result.chars().count() <= 300,
            "Should use char count for 300 limit, got {} chars",
            result.chars().count()
        );
    }
}
