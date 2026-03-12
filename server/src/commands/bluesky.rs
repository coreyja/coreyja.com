use std::path::PathBuf;

use crate::bluesky::{at_uri_to_web_url, BlueskyClient, BlueskyConfig};
use chrono::NaiveDate;
use clap::Args;
use posts::notes::FrontMatter;

/// Cutoff date - only publish notes dated on or after this date
const CUTOFF_DATE: &str = "2026-03-01";

#[derive(Args, Debug)]
pub struct PublishBlueskyArgs {
    /// Path to the note markdown file
    #[arg(long)]
    pub path: PathBuf,
}

/// Parse frontmatter from raw markdown content
fn parse_frontmatter(content: &str) -> cja::Result<(FrontMatter, String)> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return Err(cja::color_eyre::eyre::eyre!(
            "Missing frontmatter delimiter"
        ));
    }

    let rest = &content[3..];
    let Some(end_idx) = rest.find("\n---") else {
        return Err(cja::color_eyre::eyre::eyre!(
            "Missing closing frontmatter delimiter"
        ));
    };

    let yaml = &rest[..end_idx].trim();
    let body = &rest[end_idx + 4..]; // Skip "\n---"

    let frontmatter: FrontMatter = serde_yaml::from_str(yaml)
        .map_err(|e| cja::color_eyre::eyre::eyre!("Invalid YAML: {}", e))?;

    Ok((frontmatter, body.to_string()))
}

/// Update frontmatter in a markdown file with the `bsky_url`
fn update_frontmatter_with_bsky_url(content: &str, url: &str) -> String {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return content.to_string();
    }

    let rest = &content[3..];
    let Some(end_idx) = rest.find("\n---") else {
        return content.to_string();
    };

    let yaml = &rest[..end_idx];
    let body = &rest[end_idx + 4..]; // Skip "\n---"

    let updated_yaml = format!("{}\nbsky_url: {url}", yaml.trim_end());

    format!("---\n{updated_yaml}\n---{body}")
}

/// Strip markdown formatting from text for plain-text Bluesky posts
fn strip_markdown(text: &str) -> String {
    let mut result = String::with_capacity(text.len());

    for line in text.lines() {
        let line = line.trim();

        // Remove heading prefixes
        let line = if let Some(stripped) = line.strip_prefix("######") {
            stripped.trim_start()
        } else if let Some(stripped) = line.strip_prefix("#####") {
            stripped.trim_start()
        } else if let Some(stripped) = line.strip_prefix("####") {
            stripped.trim_start()
        } else if let Some(stripped) = line.strip_prefix("###") {
            stripped.trim_start()
        } else if let Some(stripped) = line.strip_prefix("##") {
            stripped.trim_start()
        } else if let Some(stripped) = line.strip_prefix('#') {
            stripped.trim_start()
        } else {
            line
        };

        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(line);
    }

    // Convert [text](url) -> text
    let mut out = String::with_capacity(result.len());
    let mut chars = result.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '[' {
            let mut link_text = String::new();
            let mut found_close = false;
            for c in chars.by_ref() {
                if c == ']' {
                    found_close = true;
                    break;
                }
                link_text.push(c);
            }
            if found_close && chars.peek() == Some(&'(') {
                chars.next(); // skip '('
                let mut depth = 1;
                for c in chars.by_ref() {
                    if c == '(' {
                        depth += 1;
                    } else if c == ')' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                }
                out.push_str(&link_text);
            } else {
                out.push('[');
                out.push_str(&link_text);
                if found_close {
                    out.push(']');
                }
            }
        } else {
            out.push(ch);
        }
    }

    // Remove inline formatting: **, *, __, _, ```, `
    out.replace("***", "")
        .replace("**", "")
        .replace("__", "")
        .replace('*', "")
        .replace("```", "")
        .replace('`', "")
}

/// Format the post text for Bluesky, truncating body if needed to stay within 300 chars
fn format_post_text(title: &str, body: &str, url: &str) -> String {
    // Format: "{title}\n\n{body}\n\n{url}"
    // The separator takes 4 chars (\n\n before body, \n\n before url)
    let overhead = title.chars().count() + url.chars().count() + 4; // \n\n + \n\n

    if overhead >= 300 {
        // Title + URL alone exceed limit, no room for body
        return format!("{title}\n\n{url}");
    }

    let max_body_chars = 300 - overhead;
    let body_trimmed = body.trim();

    if body_trimmed.chars().count() <= max_body_chars {
        format!("{title}\n\n{body_trimmed}\n\n{url}")
    } else {
        // Truncate body with ellipsis
        let truncated: String = body_trimmed
            .chars()
            .take(max_body_chars.saturating_sub(1))
            .collect();
        format!("{title}\n\n{truncated}…\n\n{url}")
    }
}

pub async fn publish_bluesky(args: &PublishBlueskyArgs) -> cja::Result<()> {
    let path = &args.path;

    let content = std::fs::read_to_string(path).map_err(|e| {
        cja::color_eyre::eyre::eyre!("Failed to read file {}: {}", path.display(), e)
    })?;

    let (frontmatter, body) = parse_frontmatter(&content)?;

    // Check if already has bsky_url (idempotency)
    if frontmatter.bsky_url.is_some() {
        println!("Note already has bsky_url, skipping: {}", path.display());
        return Ok(());
    }

    // Check cutoff date
    let cutoff =
        NaiveDate::parse_from_str(CUTOFF_DATE, "%Y-%m-%d").expect("CUTOFF_DATE should be valid");
    if frontmatter.date < cutoff {
        println!(
            "Note date {} is before cutoff date {}, skipping",
            frontmatter.date, CUTOFF_DATE
        );
        return Ok(());
    }

    // Format the post text
    let note_url = format!("https://coreyja.com/notes/{}", frontmatter.slug);
    let plain_body = strip_markdown(&body);
    let post_text = format_post_text(&frontmatter.title, &plain_body, &note_url);

    // Login and create post
    let config = BlueskyConfig::from_env()?;
    let client = BlueskyClient::login(&config).await?;

    println!("Publishing note to Bluesky: {}", frontmatter.title);
    let response = client
        .create_post(&post_text, &note_url, &frontmatter.title)
        .await?;

    let web_url = at_uri_to_web_url(&response.uri)?;
    println!("Published to Bluesky: {web_url}");

    // Update the file with bsky_url
    let updated_content = update_frontmatter_with_bsky_url(&content, &web_url);
    std::fs::write(path, updated_content).map_err(|e| {
        cja::color_eyre::eyre::eyre!("Failed to write file {}: {}", path.display(), e)
    })?;

    println!("Updated {} with bsky_url", path.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== parse_frontmatter tests ====================

    #[test]
    fn parse_frontmatter_basic_note() {
        let content = r"---
title: Test Note
date: 2026-03-05
slug: test-note
kind: til
---

This is the body of the note.
";
        let (fm, body) = parse_frontmatter(content).unwrap();
        assert_eq!(fm.title, "Test Note");
        assert_eq!(fm.slug, "test-note");
        assert!(body.contains("This is the body of the note."));
    }

    #[test]
    fn parse_frontmatter_with_bsky_url() {
        let content = r"---
title: Already Published
date: 2026-03-05
slug: already-published
kind: til
bsky_url: https://bsky.app/profile/coreyja.com/post/abc123
---

Body text.
";
        let (fm, _body) = parse_frontmatter(content).unwrap();
        assert_eq!(
            fm.bsky_url,
            Some("https://bsky.app/profile/coreyja.com/post/abc123".to_string())
        );
    }

    #[test]
    fn parse_frontmatter_without_kind() {
        let content = r"---
title: Plain Note
date: 2026-03-06
slug: plain-note
---

Just a note without a kind.
";
        let (fm, _body) = parse_frontmatter(content).unwrap();
        assert_eq!(fm.kind, None);
    }

    #[test]
    fn parse_frontmatter_missing_opening_delimiter() {
        let content = r"title: Bad
date: 2026-03-05
slug: bad
---

Body.
";
        assert!(parse_frontmatter(content).is_err());
    }

    #[test]
    fn parse_frontmatter_missing_closing_delimiter() {
        let content = r"---
title: Bad
date: 2026-03-05
slug: bad

Body without closing delimiter.
";
        assert!(parse_frontmatter(content).is_err());
    }

    // ==================== update_frontmatter_with_bsky_url tests ====================

    #[test]
    fn update_frontmatter_adds_bsky_url() {
        let content = r"---
title: Test Note
date: 2026-03-05
slug: test-note
kind: til
---

Body content here.
";
        let updated = update_frontmatter_with_bsky_url(
            content,
            "https://bsky.app/profile/coreyja.com/post/abc123",
        );
        assert!(updated.contains("bsky_url: https://bsky.app/profile/coreyja.com/post/abc123"));
        assert!(updated.contains("Body content here."));
    }

    #[test]
    fn update_frontmatter_preserves_all_existing_fields() {
        let content = r"---
title: Full Note
date: 2026-03-05
slug: full-note
kind: link
---

Body.
";
        let updated = update_frontmatter_with_bsky_url(
            content,
            "https://bsky.app/profile/coreyja.com/post/xyz",
        );

        assert!(updated.contains("title: Full Note"));
        assert!(updated.contains("date: 2026-03-05"));
        assert!(updated.contains("slug: full-note"));
        assert!(updated.contains("kind: link"));
        assert!(updated.contains("bsky_url: https://bsky.app/profile/coreyja.com/post/xyz"));
    }

    #[test]
    fn update_frontmatter_preserves_body_formatting() {
        let content = r"---
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
";
        let updated =
            update_frontmatter_with_bsky_url(content, "https://bsky.app/profile/x/post/y");

        assert!(updated.contains("# Heading"));
        assert!(updated.contains("**bold**"));
        assert!(updated.contains("- List item 1"));
        assert!(updated.contains("```rust"));
    }

    // ==================== format_post_text tests ====================

    #[test]
    fn format_post_text_basic() {
        let result = format_post_text(
            "My Title",
            "This is the body of the note.",
            "https://coreyja.com/notes/my-note",
        );
        assert!(result.contains("My Title"));
        assert!(result.contains("This is the body"));
        assert!(result.contains("https://coreyja.com/notes/my-note"));
    }

    #[test]
    fn format_post_text_structure() {
        let result = format_post_text("Title", "Body text", "https://coreyja.com/notes/slug");
        let lines: Vec<&str> = result.split('\n').collect();
        assert_eq!(lines[0], "Title");
        assert_eq!(lines[1], "");
        assert!(result.ends_with("https://coreyja.com/notes/slug"));
    }

    #[test]
    fn format_post_text_under_300_chars_no_truncation() {
        let title = "Short Title";
        let body = "Short body.";
        let url = "https://coreyja.com/notes/short";
        let result = format_post_text(title, body, url);

        assert!(result.chars().count() <= 300);
        assert!(result.contains("Short body."));
    }

    #[test]
    fn format_post_text_truncates_body_when_over_300_chars() {
        let title = "Title";
        let body = "A".repeat(400);
        let url = "https://coreyja.com/notes/long-note";
        let result = format_post_text(title, &body, url);

        assert!(
            result.chars().count() <= 300,
            "got {} chars",
            result.chars().count()
        );
        assert!(result.contains("Title"));
        assert!(result.contains(url));
    }

    #[test]
    fn format_post_text_preserves_title_and_url_when_truncating() {
        let title = "My Important Title";
        let body = "x".repeat(500);
        let url = "https://coreyja.com/notes/important";
        let result = format_post_text(title, &body, url);

        assert!(result.starts_with("My Important Title"));
        assert!(result.ends_with(url));
        assert!(result.chars().count() <= 300);
    }

    // ==================== strip_markdown tests ====================

    #[test]
    fn strip_markdown_removes_bold() {
        assert_eq!(strip_markdown("Hello **world**"), "Hello world");
    }

    #[test]
    fn strip_markdown_removes_italic_star() {
        assert_eq!(strip_markdown("Hello *world*"), "Hello world");
    }

    #[test]
    fn strip_markdown_removes_underscores() {
        assert_eq!(strip_markdown("Hello __world__"), "Hello world");
    }

    #[test]
    fn strip_markdown_removes_backticks() {
        assert_eq!(strip_markdown("Use `code` here"), "Use code here");
    }

    #[test]
    fn strip_markdown_removes_code_blocks() {
        let result = strip_markdown("```rust\nfn main() {}\n```");
        assert!(result.contains("fn main() {}"));
        assert!(!result.contains("```"));
    }

    #[test]
    fn strip_markdown_removes_heading_prefixes() {
        assert_eq!(strip_markdown("# Heading"), "Heading");
        assert_eq!(strip_markdown("## Subheading"), "Subheading");
        assert_eq!(strip_markdown("### Third"), "Third");
    }

    #[test]
    fn strip_markdown_converts_links_to_text() {
        assert_eq!(
            strip_markdown("Check [MDN docs](https://developer.mozilla.org/) for details"),
            "Check MDN docs for details"
        );
    }

    #[test]
    fn strip_markdown_handles_nested_parens_in_urls() {
        assert_eq!(
            strip_markdown("[wiki](https://en.wikipedia.org/wiki/Rust_(programming_language))"),
            "wiki"
        );
    }

    #[test]
    fn strip_markdown_passes_through_plain_text() {
        assert_eq!(strip_markdown("Just plain text"), "Just plain text");
    }

    // ==================== format_post_text tests (continued) ====================

    #[test]
    fn format_post_text_uses_char_count_not_byte_count() {
        let title = "🦀 Rust";
        let body = "é".repeat(300);
        let url = "https://coreyja.com/notes/emoji";
        let result = format_post_text(title, &body, url);

        assert!(
            result.chars().count() <= 300,
            "got {} chars",
            result.chars().count()
        );
    }
}
