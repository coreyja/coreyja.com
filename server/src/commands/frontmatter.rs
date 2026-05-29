//! Generic YAML frontmatter helpers shared across post-type CLI commands.
//!
//! These intentionally operate on raw `&str` so callers can deserialize the
//! YAML into whichever concrete frontmatter type they need (note vs blog).

/// Split a markdown file into `(yaml, body)` without binding to a concrete type.
/// Returns `Err` if the document is missing `---` delimiters.
pub fn split_frontmatter(content: &str) -> cja::Result<(&str, &str)> {
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

    let yaml = rest[..end_idx].trim();
    let body = &rest[end_idx + 4..]; // Skip "\n---"

    Ok((yaml, body))
}

/// Append `key: value` lines to the YAML frontmatter. Values written verbatim —
/// safe for AT URIs and bsky URLs which contain no YAML-special chars.
/// Returns input unchanged if document has no frontmatter.
pub fn append_frontmatter_keys(content: &str, kv: &[(&str, &str)]) -> String {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return content.to_string();
    }

    let rest = &trimmed[3..];
    let Some(end_idx) = rest.find("\n---") else {
        return content.to_string();
    };

    let yaml = &rest[..end_idx];
    let body = &rest[end_idx + 4..]; // Skip "\n---"

    let mut updated_yaml = yaml.trim_end().to_string();
    for (key, value) in kv {
        updated_yaml.push('\n');
        updated_yaml.push_str(key);
        updated_yaml.push_str(": ");
        updated_yaml.push_str(value);
    }

    format!("---\n{updated_yaml}\n---{body}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_frontmatter_basic() {
        let content = "---\ntitle: T\ndate: 2026-05-01\n---\n\nbody text\n";
        let (yaml, body) = split_frontmatter(content).unwrap();
        assert!(yaml.contains("title: T"));
        assert!(yaml.contains("date: 2026-05-01"));
        assert!(body.contains("body text"));
    }

    #[test]
    fn split_frontmatter_missing_opening_delimiter() {
        let content = "title: T\n---\n\nbody\n";
        assert!(split_frontmatter(content).is_err());
    }

    #[test]
    fn split_frontmatter_missing_closing_delimiter() {
        let content = "---\ntitle: T\n\nbody without closing\n";
        assert!(split_frontmatter(content).is_err());
    }

    #[test]
    fn split_frontmatter_preserves_body_whitespace() {
        let content = "---\ntitle: T\n---\n\n\n# Heading\n\nParagraph.\n";
        let (_, body) = split_frontmatter(content).unwrap();
        // The body starts after "\n---", so two leading newlines + body.
        assert!(body.starts_with("\n\n\n# Heading"));
        assert!(body.contains("Paragraph."));
    }

    #[test]
    fn append_single_key_appends_correctly() {
        let content = "---\ntitle: T\ndate: 2026-05-01\n---\n\nbody\n";
        let updated = append_frontmatter_keys(content, &[("bsky_url", "https://bsky.app/x")]);
        assert!(updated.contains("title: T"));
        assert!(updated.contains("date: 2026-05-01"));
        assert!(updated.contains("bsky_url: https://bsky.app/x"));
        assert!(updated.contains("body"));
    }

    #[test]
    fn append_multiple_keys_adds_them_in_order() {
        let content = "---\ntitle: T\n---\n\nbody\n";
        let updated = append_frontmatter_keys(
            content,
            &[
                ("atproto_uri", "at://abc"),
                ("bsky_url", "https://bsky.app/x"),
            ],
        );
        let at_idx = updated.find("atproto_uri:").unwrap();
        let bsky_idx = updated.find("bsky_url:").unwrap();
        assert!(at_idx < bsky_idx);
    }

    #[test]
    fn append_keys_no_frontmatter_returns_input_unchanged() {
        let content = "no frontmatter here\n";
        let updated = append_frontmatter_keys(content, &[("k", "v")]);
        assert_eq!(updated, content);
    }

    #[test]
    fn append_keys_missing_closing_returns_input_unchanged() {
        let content = "---\ntitle: T\n\nbody no closing\n";
        let updated = append_frontmatter_keys(content, &[("k", "v")]);
        assert_eq!(updated, content);
    }

    #[test]
    fn append_keys_preserves_body_formatting() {
        let content = "---\ntitle: T\n---\n\n# Heading\n\n- list\n- items\n";
        let updated = append_frontmatter_keys(content, &[("k", "v")]);
        assert!(updated.contains("# Heading"));
        assert!(updated.contains("- list"));
        assert!(updated.contains("- items"));
        assert!(updated.contains("k: v"));
    }
}
