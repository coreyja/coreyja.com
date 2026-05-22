use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use clap::Args;
use posts::notes::FrontMatter;
use crate::bluesky::{at_uri_to_web_url, BlueskyClient, BlueskyConfig};

/// Cutoff date - only publish notes dated on or after this date
const CUTOFF_DATE: &str = "2026-03-01";

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
pub struct PublishBlueskyArgs {
    /// Path to a single note markdown file
    #[arg(long, group = "input")]
    pub path: Option<PathBuf>,

    /// Directory to scan for unpublished notes (notes/*.md)
    #[arg(long, group = "input")]
    pub dir: Option<PathBuf>,
}

fn cutoff_date() -> NaiveDate {
    NaiveDate::parse_from_str(CUTOFF_DATE, "%Y-%m-%d").expect("CUTOFF_DATE should be valid")
}

/// Classify a note file: should it be published?
///
/// Returns `Ok(Some(frontmatter))` when the note is eligible for publishing,
/// `Ok(None)` when it should be skipped (already published or before cutoff),
/// and `Err` when the file can't be read or parsed.
fn classify_note(path: &Path) -> cja::Result<Option<FrontMatter>> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        cja::color_eyre::eyre::eyre!("Failed to read file {}: {}", path.display(), e)
    })?;

    // Cheap idempotency check before YAML parse.
    if content.contains("bsky_url:") {
        return Ok(None);
    }

    let (frontmatter, _body) = parse_frontmatter(&content)?;

    if frontmatter.date < cutoff_date() {
        return Ok(None);
    }

    Ok(Some(frontmatter))
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

/// Publish a single note, given an authenticated client. Idempotent: skips
/// notes that already have a `bsky_url` or are dated before the cutoff.
async fn publish_one(path: &Path, client: &BlueskyClient) -> cja::Result<bool> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        cja::color_eyre::eyre::eyre!("Failed to read file {}: {}", path.display(), e)
    })?;

    if content.contains("bsky_url:") {
        println!("Note already has bsky_url, skipping: {}", path.display());
        return Ok(false);
    }

    let (frontmatter, body) = parse_frontmatter(&content)?;

    if frontmatter.date < cutoff_date() {
        println!(
            "Note date {} is before cutoff date {}, skipping",
            frontmatter.date, CUTOFF_DATE
        );
        return Ok(false);
    }

    let note_url = format!("https://coreyja.com/notes/{}", frontmatter.slug);

    println!("Publishing note to Bluesky: {}", frontmatter.title);
    let response = client
        .create_note_post(&frontmatter.title, &body, &note_url)
        .await?;

    let web_url = at_uri_to_web_url(&response.uri)?;
    println!("Published to Bluesky: {web_url}");

    let updated_content = update_frontmatter_with_bsky_url(&content, &web_url);
    std::fs::write(path, updated_content).map_err(|e| {
        cja::color_eyre::eyre::eyre!("Failed to write file {}: {}", path.display(), e)
    })?;

    println!("Updated {} with bsky_url", path.display());
    Ok(true)
}

/// Scan a directory for `.md` notes that are eligible for publishing.
fn find_unpublished_notes(dir: &Path) -> cja::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|e| {
        cja::color_eyre::eyre::eyre!("Failed to read directory {}: {}", dir.display(), e)
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            cja::color_eyre::eyre::eyre!("Failed to read directory entry: {}", e)
        })?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        match classify_note(&path) {
            Ok(Some(_)) => paths.push(path),
            Ok(None) => {}
            Err(e) => {
                // Surface but don't abort — a single malformed note shouldn't
                // block syndication of the rest.
                eprintln!("Failed to classify {}: {}", path.display(), e);
            }
        }
    }

    paths.sort();
    Ok(paths)
}

pub async fn publish_bluesky(args: &PublishBlueskyArgs) -> cja::Result<()> {
    let config = BlueskyConfig::from_env()?;
    let client = BlueskyClient::login(&config).await?;

    match (&args.path, &args.dir) {
        (Some(path), None) => {
            publish_one(path, &client).await?;
            Ok(())
        }
        (None, Some(dir)) => {
            let paths = find_unpublished_notes(dir)?;
            if paths.is_empty() {
                println!("No unpublished notes found in {}", dir.display());
                return Ok(());
            }

            println!("Found {} note(s) to publish", paths.len());
            let mut failures: Vec<(PathBuf, cja::color_eyre::Report)> = Vec::new();
            let mut published = 0;
            for path in &paths {
                match publish_one(path, &client).await {
                    Ok(true) => published += 1,
                    Ok(false) => {}
                    Err(e) => {
                        eprintln!("Failed to publish {}: {}", path.display(), e);
                        failures.push((path.clone(), e));
                    }
                }
            }

            println!(
                "Done. Published: {published}, skipped: {}, failed: {}",
                paths.len() - published - failures.len(),
                failures.len()
            );

            if !failures.is_empty() {
                return Err(cja::color_eyre::eyre::eyre!(
                    "{} note(s) failed to publish (see logs above); next run will retry",
                    failures.len()
                ));
            }

            Ok(())
        }
        // clap's ArgGroup with required=true, multiple=false guarantees we
        // never land here, but keep an explicit branch for clarity.
        _ => Err(cja::color_eyre::eyre::eyre!(
            "exactly one of --path or --dir must be provided"
        )),
    }
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

    // ==================== classify_note / scan tests ====================

    fn write_note(dir: &std::path::Path, name: &str, body: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, body).expect("write test note");
        path
    }

    #[test]
    fn classify_note_skips_already_published() {
        let tmp = tempfile::tempdir().unwrap();
        let path = write_note(
            tmp.path(),
            "a.md",
            "---\ntitle: A\ndate: 2026-03-05\nslug: a\nbsky_url: https://bsky.app/x\n---\n\nbody\n",
        );
        assert!(classify_note(&path).unwrap().is_none());
    }

    #[test]
    fn classify_note_skips_before_cutoff() {
        let tmp = tempfile::tempdir().unwrap();
        let path = write_note(
            tmp.path(),
            "old.md",
            "---\ntitle: Old\ndate: 2024-01-01\nslug: old\n---\n\nbody\n",
        );
        assert!(classify_note(&path).unwrap().is_none());
    }

    #[test]
    fn classify_note_accepts_eligible_note() {
        let tmp = tempfile::tempdir().unwrap();
        let path = write_note(
            tmp.path(),
            "new.md",
            "---\ntitle: New\ndate: 2026-04-15\nslug: new\n---\n\nbody\n",
        );
        let fm = classify_note(&path).unwrap().expect("should be eligible");
        assert_eq!(fm.slug, "new");
    }

    #[test]
    fn classify_note_accepts_cutoff_day_boundary() {
        let tmp = tempfile::tempdir().unwrap();
        let path = write_note(
            tmp.path(),
            "boundary.md",
            "---\ntitle: Boundary\ndate: 2026-03-01\nslug: boundary\n---\n\nbody\n",
        );
        assert!(classify_note(&path).unwrap().is_some());
    }

    #[test]
    fn find_unpublished_notes_returns_only_eligible_md_files() {
        let tmp = tempfile::tempdir().unwrap();
        write_note(
            tmp.path(),
            "published.md",
            "---\ntitle: P\ndate: 2026-04-01\nslug: p\nbsky_url: https://bsky.app/x\n---\n\nbody\n",
        );
        write_note(
            tmp.path(),
            "old.md",
            "---\ntitle: O\ndate: 2024-01-01\nslug: o\n---\n\nbody\n",
        );
        let eligible = write_note(
            tmp.path(),
            "fresh.md",
            "---\ntitle: F\ndate: 2026-04-15\nslug: f\n---\n\nbody\n",
        );
        // Non-markdown files should be ignored.
        write_note(tmp.path(), "readme.txt", "not a note");

        let found = find_unpublished_notes(tmp.path()).unwrap();
        assert_eq!(found, vec![eligible]);
    }

    #[test]
    fn find_unpublished_notes_skips_malformed_files() {
        let tmp = tempfile::tempdir().unwrap();
        // Missing frontmatter — should be logged and skipped, not abort the scan.
        write_note(tmp.path(), "bad.md", "no frontmatter here\n");
        let eligible = write_note(
            tmp.path(),
            "good.md",
            "---\ntitle: G\ndate: 2026-04-15\nslug: g\n---\n\nbody\n",
        );

        let found = find_unpublished_notes(tmp.path()).unwrap();
        assert_eq!(found, vec![eligible]);
    }

}
