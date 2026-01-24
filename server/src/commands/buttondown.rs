use std::path::{Path, PathBuf};

use chrono::{NaiveDate, Utc};
use clap::Args;
use posts::blog::BlogFrontMatter;

use crate::buttondown::{ButtondownClient, ButtondownConfig, CreateEmailRequest, EmailStatus};

/// Cutoff date - only publish newsletters dated on or after this date
const CUTOFF_DATE: &str = "2026-01-25";

/// Base URL for the blog
const BLOG_BASE_URL: &str = "https://coreyja.com/blog";

#[derive(Args, Debug)]
pub struct PublishButtondownArgs {
    /// Path to the newsletter markdown file
    #[arg(long)]
    pub path: PathBuf,
}

/// Rewrite relative image URLs to absolute URLs
///
/// Transforms `./image.png` to `https://coreyja.com/blog/weekly/20260123/image.png`
fn rewrite_image_urls(content: &str, post_dir: &str) -> String {
    // Build the base URL for this post's images
    let base_url = format!("{BLOG_BASE_URL}/{post_dir}");

    // Replace ./path with absolute URL
    // This handles the common case of `![alt](./image.png)`
    content.replace("](./", &format!("]({base_url}/"))
}

/// Extract the directory path from a file path for URL construction
///
/// e.g., `blog/weekly/20260123/index.md` -> `weekly/20260123`
fn extract_post_dir(path: &Path) -> cja::Result<String> {
    let path_str = path.to_string_lossy();

    // Find the blog/ prefix and extract the rest
    if let Some(idx) = path_str.find("blog/") {
        let after_blog = &path_str[idx + 5..]; // Skip "blog/"

        // Remove the filename (index.md or whatever.md)
        if let Some(parent) = std::path::Path::new(after_blog).parent() {
            return Ok(parent.to_string_lossy().to_string());
        }
    }

    Err(cja::color_eyre::eyre::eyre!(
        "Could not extract post directory from path: {}",
        path_str
    ))
}

/// Parse frontmatter from raw markdown content
fn parse_frontmatter(content: &str) -> cja::Result<(BlogFrontMatter, String)> {
    // Find the frontmatter delimiters
    let content = content.trim_start();
    if !content.starts_with("---") {
        return Err(cja::color_eyre::eyre::eyre!("Missing frontmatter delimiter"));
    }

    // Find the closing delimiter
    let rest = &content[3..];
    let Some(end_idx) = rest.find("\n---") else {
        return Err(cja::color_eyre::eyre::eyre!(
            "Missing closing frontmatter delimiter"
        ));
    };

    let yaml = &rest[..end_idx].trim();
    let body = &rest[end_idx + 4..]; // Skip "\n---"

    let frontmatter: BlogFrontMatter =
        serde_yaml::from_str(yaml).map_err(|e| cja::color_eyre::eyre::eyre!("Invalid YAML: {}", e))?;

    Ok((frontmatter, body.to_string()))
}

/// Update frontmatter in a markdown file with the `buttondown_id`
fn update_frontmatter_with_id(content: &str, buttondown_id: &str) -> cja::Result<String> {
    // Find the frontmatter delimiters
    let content = content.trim_start();
    if !content.starts_with("---") {
        return Err(cja::color_eyre::eyre::eyre!("Missing frontmatter delimiter"));
    }

    // Find the closing delimiter
    let rest = &content[3..];
    let Some(end_idx) = rest.find("\n---") else {
        return Err(cja::color_eyre::eyre::eyre!(
            "Missing closing frontmatter delimiter"
        ));
    };

    let yaml = &rest[..end_idx];
    let body = &rest[end_idx + 4..]; // Skip "\n---"

    // Add buttondown_id to the frontmatter
    let updated_yaml = format!("{}\nbuttondown_id: {buttondown_id}", yaml.trim_end());

    Ok(format!("---\n{updated_yaml}\n---{body}"))
}

pub async fn publish_buttondown(args: &PublishButtondownArgs) -> cja::Result<()> {
    let path = &args.path;

    // Read the file
    let content = std::fs::read_to_string(path)
        .map_err(|e| cja::color_eyre::eyre::eyre!("Failed to read file {}: {}", path.display(), e))?;

    // Parse frontmatter
    let (frontmatter, body) = parse_frontmatter(&content)?;

    // Validate this is a newsletter
    if !frontmatter.is_newsletter {
        return Err(cja::color_eyre::eyre::eyre!(
            "Post is not marked as a newsletter (is_newsletter: false)"
        ));
    }

    // Check if already published
    if let Some(id) = &frontmatter.buttondown_id {
        println!("Newsletter already published to Buttondown (id: {id})");
        return Ok(());
    }

    // Check cutoff date
    let cutoff = NaiveDate::parse_from_str(CUTOFF_DATE, "%Y-%m-%d")
        .expect("CUTOFF_DATE should be valid");
    if frontmatter.date < cutoff {
        println!(
            "Newsletter date {} is before cutoff date {}, skipping",
            frontmatter.date, CUTOFF_DATE
        );
        return Ok(());
    }

    // Extract post directory for URL rewriting
    let post_dir = extract_post_dir(path)?;

    // Rewrite image URLs
    let body_with_absolute_urls = rewrite_image_urls(&body, &post_dir);

    // Determine status based on newsletter_send_at
    let (status, publish_date) = match frontmatter.newsletter_send_at {
        Some(send_at) => {
            // Check if the scheduled time is in the future
            if send_at <= Utc::now() {
                println!("Warning: newsletter_send_at ({send_at}) is in the past, sending immediately");
                (EmailStatus::AboutToSend, None)
            } else {
                (EmailStatus::Scheduled, Some(send_at))
            }
        }
        None => (EmailStatus::AboutToSend, None),
    };

    // Create the request
    let request = CreateEmailRequest {
        subject: frontmatter.title.clone(),
        body: body_with_absolute_urls,
        status,
        publish_date,
    };

    // Load config and create client
    let config = ButtondownConfig::from_env()?;
    let client = ButtondownClient::new(&config);

    // Send the request
    println!("Publishing newsletter: {}", frontmatter.title);
    let response = client.create_email(&request).await?;

    println!(
        "Successfully published to Buttondown with id: {}",
        response.id
    );

    // Update the file with the buttondown_id
    let updated_content = update_frontmatter_with_id(&content, &response.id)?;
    std::fs::write(path, updated_content)
        .map_err(|e| cja::color_eyre::eyre::eyre!("Failed to write file {}: {}", path.display(), e))?;

    println!("Updated {} with buttondown_id", path.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_post_dir() {
        let path = PathBuf::from("blog/weekly/20260123/index.md");
        assert_eq!(extract_post_dir(&path).unwrap(), "weekly/20260123");

        let path = PathBuf::from("/home/user/code/coreyja.com/blog/weekly/20260123/index.md");
        assert_eq!(extract_post_dir(&path).unwrap(), "weekly/20260123");
    }

    #[test]
    fn test_rewrite_image_urls() {
        let content = "![alt text](./image.png)\nSome text\n![another](./path/to/image.jpg)";
        let rewritten = rewrite_image_urls(content, "weekly/20260123");

        assert!(rewritten.contains("](https://coreyja.com/blog/weekly/20260123/image.png)"));
        assert!(rewritten.contains("](https://coreyja.com/blog/weekly/20260123/path/to/image.jpg)"));
        assert!(!rewritten.contains("](./"));
    }

    #[test]
    fn test_parse_frontmatter() {
        let content = r"---
title: Test Newsletter
date: 2026-01-25
is_newsletter: true
---

This is the body.
";
        let (fm, body) = parse_frontmatter(content).unwrap();
        assert_eq!(fm.title, "Test Newsletter");
        assert!(fm.is_newsletter);
        assert!(body.contains("This is the body."));
    }

    #[test]
    fn test_update_frontmatter_with_id() {
        let content = r"---
title: Test Newsletter
date: 2026-01-25
is_newsletter: true
---

Body content here.
";
        let updated = update_frontmatter_with_id(content, "abc123").unwrap();
        assert!(updated.contains("buttondown_id: abc123"));
        assert!(updated.contains("Body content here."));
    }
}
