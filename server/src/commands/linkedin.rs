#![allow(clippy::doc_markdown)]

use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use clap::Args;
use posts::{blog::BlogFrontMatter, podcast::PodcastFrontMatter};

use crate::{
    linkedin::{
        compose_linkedin_body, extract_first_paragraph, linkedin_urn_to_web_url, LinkedInClient,
    },
    AppConfig,
};

/// Cutoff date — confirm/update at PR-open time; must be the actual merge day.
const LINKEDIN_CUTOFF_DATE: &str = "2026-05-23";

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum LinkedInKind {
    Blog,
    Newsletter,
    Podcast,
}

#[derive(Args, Debug)]
pub struct PublishLinkedInArgs {
    #[arg(long)]
    pub dir: PathBuf,
    #[arg(long, value_enum)]
    pub kind: LinkedInKind,
}

fn cutoff_date() -> NaiveDate {
    NaiveDate::parse_from_str(LINKEDIN_CUTOFF_DATE, "%Y-%m-%d")
        .expect("LINKEDIN_CUTOFF_DATE should be valid")
}

pub async fn publish_linkedin(args: &PublishLinkedInArgs) -> cja::Result<()> {
    let app_config = AppConfig::from_env()?;
    let client = LinkedInClient::from_db_env().await?;

    let paths = match args.kind {
        LinkedInKind::Blog => find_blog_posts(&args.dir)?,
        LinkedInKind::Newsletter => find_newsletters(&args.dir)?,
        LinkedInKind::Podcast => find_podcast_episodes(&args.dir)?,
    };

    if paths.is_empty() {
        println!("No unpublished items found in {}", args.dir.display());
        return Ok(());
    }

    println!("Found {} item(s) to publish", paths.len());

    let mut failures: Vec<(PathBuf, cja::color_eyre::Report)> = Vec::new();
    let mut published = 0usize;
    for path in &paths {
        let result = match args.kind {
            LinkedInKind::Blog => publish_one_blog(path, &client, &app_config).await,
            LinkedInKind::Newsletter => publish_one_newsletter(path, &client, &app_config).await,
            LinkedInKind::Podcast => publish_one_podcast(path, &client, &app_config).await,
        };
        match result {
            Ok(true) => published += 1,
            Ok(false) => {}
            Err(e) => {
                eprintln!("Failed to publish {}: {e}", path.display());
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
            "{} item(s) failed to publish (see logs above); next run will retry",
            failures.len()
        ));
    }

    Ok(())
}

// ==================== Scanners ====================

fn find_blog_posts(dir: &Path) -> cja::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|e| {
        cja::color_eyre::eyre::eyre!("Failed to read directory {}: {}", dir.display(), e)
    })?;

    for entry in entries {
        let entry = entry?;
        let p = entry.path();
        if !p.is_dir() {
            continue;
        }
        let index = p.join("index.md");
        if !index.is_file() {
            continue;
        }
        match classify_blog(&index) {
            Ok(Some(())) => paths.push(index),
            Ok(None) => {}
            Err(e) => eprintln!("Failed to classify {}: {e}", index.display()),
        }
    }
    paths.sort();
    Ok(paths)
}

fn find_newsletters(dir: &Path) -> cja::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|e| {
        cja::color_eyre::eyre::eyre!("Failed to read directory {}: {}", dir.display(), e)
    })?;

    for entry in entries {
        let entry = entry?;
        let p = entry.path();
        if !p.is_dir() {
            continue;
        }
        let index = p.join("index.md");
        if !index.is_file() {
            continue;
        }
        match classify_newsletter(&index) {
            Ok(Some(())) => paths.push(index),
            Ok(None) => {}
            Err(e) => eprintln!("Failed to classify {}: {e}", index.display()),
        }
    }
    paths.sort();
    Ok(paths)
}

fn find_podcast_episodes(dir: &Path) -> cja::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|e| {
        cja::color_eyre::eyre::eyre!("Failed to read directory {}: {}", dir.display(), e)
    })?;

    for entry in entries {
        let entry = entry?;
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        // Skip sibling LinkedIn custom-body files.
        if name == "linkedin.md" || name.ends_with(".linkedin.md") {
            continue;
        }
        if p.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        match classify_podcast(&p) {
            Ok(Some(())) => paths.push(p),
            Ok(None) => {}
            Err(e) => eprintln!("Failed to classify {}: {e}", p.display()),
        }
    }
    paths.sort();
    Ok(paths)
}

// ==================== Classifiers ====================

/// `Ok(Some(()))` = eligible; `Ok(None)` = skip; `Err` = parse/read failure.
fn classify_blog(path: &Path) -> cja::Result<Option<()>> {
    let content = read_to_string(path)?;
    if content.contains("linkedin_url:") {
        return Ok(None);
    }
    let (fm, _) = parse_blog_frontmatter(&content)?;
    if fm.is_newsletter {
        return Ok(None);
    }
    if fm.linkedin_url.is_some() {
        return Ok(None);
    }
    if fm.date < cutoff_date() {
        return Ok(None);
    }
    Ok(Some(()))
}

fn classify_newsletter(path: &Path) -> cja::Result<Option<()>> {
    let content = read_to_string(path)?;
    if content.contains("linkedin_url:") {
        return Ok(None);
    }
    let (fm, _) = parse_blog_frontmatter(&content)?;
    if !fm.is_newsletter {
        return Ok(None);
    }
    if fm.linkedin_url.is_some() {
        return Ok(None);
    }
    if fm.date < cutoff_date() {
        return Ok(None);
    }
    Ok(Some(()))
}

fn classify_podcast(path: &Path) -> cja::Result<Option<()>> {
    let content = read_to_string(path)?;
    if content.contains("linkedin_url:") {
        return Ok(None);
    }
    let (fm, _) = parse_podcast_frontmatter(&content)?;
    if fm.linkedin_url.is_some() {
        return Ok(None);
    }
    if fm.date < cutoff_date() {
        return Ok(None);
    }
    Ok(Some(()))
}

// ==================== Publishers ====================

async fn publish_one_blog(
    path: &Path,
    client: &LinkedInClient,
    app_config: &AppConfig,
) -> cja::Result<bool> {
    let content = read_to_string(path)?;
    let (fm, body) = parse_blog_frontmatter(&content)?;
    if fm.linkedin_url.is_some() {
        println!("Already syndicated, skipping: {}", path.display());
        return Ok(false);
    }
    let custom = read_custom_body(path, fm.linkedin_content.as_deref());
    let first_para = match custom {
        Some(s) => s,
        None => extract_first_paragraph(&body),
    };
    let url = canonical_url_for_blog(path, app_config)?;
    publish_and_write(path, client, &fm.title, &first_para, &url).await
}

async fn publish_one_newsletter(
    path: &Path,
    client: &LinkedInClient,
    app_config: &AppConfig,
) -> cja::Result<bool> {
    let content = read_to_string(path)?;
    let (fm, body) = parse_blog_frontmatter(&content)?;
    if fm.linkedin_url.is_some() {
        println!("Already syndicated, skipping: {}", path.display());
        return Ok(false);
    }
    let custom = read_custom_body(path, fm.linkedin_content.as_deref());
    let first_para = match custom {
        Some(s) => s,
        None => extract_first_paragraph(&body),
    };
    let url = canonical_url_for_newsletter(path, app_config)?;
    publish_and_write(path, client, &fm.title, &first_para, &url).await
}

async fn publish_one_podcast(
    path: &Path,
    client: &LinkedInClient,
    app_config: &AppConfig,
) -> cja::Result<bool> {
    let content = read_to_string(path)?;
    let (fm, body) = parse_podcast_frontmatter(&content)?;
    if fm.linkedin_url.is_some() {
        println!("Already syndicated, skipping: {}", path.display());
        return Ok(false);
    }
    let custom = read_custom_body(path, fm.linkedin_content.as_deref());
    let first_para = match custom {
        Some(s) => s,
        None => extract_first_paragraph(&body),
    };
    let url = canonical_url_for_podcast(&fm, app_config);
    publish_and_write(path, client, &fm.title, &first_para, &url).await
}

async fn publish_and_write(
    path: &Path,
    client: &LinkedInClient,
    title: &str,
    body: &str,
    canonical_url: &str,
) -> cja::Result<bool> {
    let commentary = compose_linkedin_body(body, title, canonical_url);
    println!("Publishing to LinkedIn: {title}");
    let resp = client.create_text_post(&commentary).await?;
    let web_url = linkedin_urn_to_web_url(&resp.urn)?;
    println!("Published to LinkedIn: {web_url}");

    // Re-read the file from disk and confirm linkedin_url is still absent.
    let latest = read_to_string(path)?;
    if latest.contains("linkedin_url:") {
        tracing::warn!(
            "Race detected: {} acquired linkedin_url between scan and write — skipping write",
            path.display()
        );
        return Ok(false);
    }

    let updated = update_frontmatter_with_linkedin_url(&latest, &web_url)?;
    std::fs::write(path, updated).map_err(|e| {
        cja::color_eyre::eyre::eyre!("Failed to write file {}: {}", path.display(), e)
    })?;
    println!("Updated {} with linkedin_url", path.display());
    Ok(true)
}

// ==================== Helpers ====================

fn read_to_string(path: &Path) -> cja::Result<String> {
    std::fs::read_to_string(path)
        .map_err(|e| cja::color_eyre::eyre::eyre!("Failed to read file {}: {}", path.display(), e))
}

/// Sibling-file path for a custom LinkedIn body.
/// - `blog/<x>/index.md` and `blog/weekly/<x>/index.md` → `<parent>/linkedin.md`
/// - `podcast/<slug>.md` → `<parent>/<slug>.linkedin.md`
fn sibling_linkedin_path_for(post_path: &Path) -> PathBuf {
    let file_name = post_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let parent = post_path.parent().unwrap_or_else(|| Path::new("."));
    if file_name == "index.md" {
        parent.join("linkedin.md")
    } else if let Some(stem) = post_path.file_stem().and_then(|s| s.to_str()) {
        parent.join(format!("{stem}.linkedin.md"))
    } else {
        parent.join("linkedin.md")
    }
}

/// First non-empty source wins, in this order:
/// 1. Sibling `linkedin.md` file (after `trim()`)
/// 2. `frontmatter_linkedin_content` field (after `trim()`)
///
/// Returns `None` to signal "fall back to first-paragraph extraction".
fn read_custom_body(
    post_path: &Path,
    frontmatter_linkedin_content: Option<&str>,
) -> Option<String> {
    let sibling = sibling_linkedin_path_for(post_path);
    if let Ok(s) = std::fs::read_to_string(&sibling) {
        if !s.trim().is_empty() {
            return Some(s);
        }
    }
    if let Some(s) = frontmatter_linkedin_content {
        if !s.trim().is_empty() {
            return Some(s.to_string());
        }
    }
    None
}

/// Append `linkedin_url: <url>` into the YAML frontmatter block. Returns
/// `Err` on malformed frontmatter or if `linkedin_url:` is already present
/// (defense-in-depth: never silently no-op).
fn update_frontmatter_with_linkedin_url(content: &str, url: &str) -> cja::Result<String> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Err(cja::color_eyre::eyre::eyre!(
            "Missing frontmatter delimiter"
        ));
    }
    let rest = &trimmed[3..];
    let Some(end_idx) = rest.find("\n---") else {
        return Err(cja::color_eyre::eyre::eyre!(
            "Missing closing frontmatter delimiter"
        ));
    };
    let yaml = &rest[..end_idx];
    let body = &rest[end_idx + 4..];

    if yaml.contains("linkedin_url:") {
        return Err(cja::color_eyre::eyre::eyre!(
            "linkedin_url already present in frontmatter"
        ));
    }

    let updated_yaml = format!("{}\nlinkedin_url: {url}", yaml.trim_end());
    Ok(format!("---\n{updated_yaml}\n---{body}"))
}

fn parse_blog_frontmatter(content: &str) -> cja::Result<(BlogFrontMatter, String)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Err(cja::color_eyre::eyre::eyre!(
            "Missing frontmatter delimiter"
        ));
    }
    let rest = &trimmed[3..];
    let Some(end_idx) = rest.find("\n---") else {
        return Err(cja::color_eyre::eyre::eyre!(
            "Missing closing frontmatter delimiter"
        ));
    };
    let yaml = rest[..end_idx].trim();
    let body = &rest[end_idx + 4..];
    let fm: BlogFrontMatter = serde_yaml::from_str(yaml)
        .map_err(|e| cja::color_eyre::eyre::eyre!("Invalid YAML: {e}"))?;
    Ok((fm, body.to_string()))
}

fn parse_podcast_frontmatter(content: &str) -> cja::Result<(PodcastFrontMatter, String)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Err(cja::color_eyre::eyre::eyre!(
            "Missing frontmatter delimiter"
        ));
    }
    let rest = &trimmed[3..];
    let Some(end_idx) = rest.find("\n---") else {
        return Err(cja::color_eyre::eyre::eyre!(
            "Missing closing frontmatter delimiter"
        ));
    };
    let yaml = rest[..end_idx].trim();
    let body = &rest[end_idx + 4..];
    let fm: PodcastFrontMatter = serde_yaml::from_str(yaml)
        .map_err(|e| cja::color_eyre::eyre::eyre!("Invalid YAML: {e}"))?;
    Ok((fm, body.to_string()))
}

// ==================== Canonical URLs ====================

fn canonical_url_for_blog(post_path: &Path, app_config: &AppConfig) -> cja::Result<String> {
    // blog/<slug>/index.md → /posts/<slug>/
    let slug = post_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .ok_or_else(|| {
            cja::color_eyre::eyre::eyre!(
                "Could not derive blog slug from path: {}",
                post_path.display()
            )
        })?;
    Ok(app_config.app_url(&format!("/posts/{slug}/")))
}

fn canonical_url_for_newsletter(post_path: &Path, app_config: &AppConfig) -> cja::Result<String> {
    // blog/weekly/<YYYYMMDD>/index.md → /posts/weekly/<YYYYMMDD>/
    let dir = post_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .ok_or_else(|| {
            cja::color_eyre::eyre::eyre!(
                "Could not derive newsletter date from path: {}",
                post_path.display()
            )
        })?;
    Ok(app_config.app_url(&format!("/posts/weekly/{dir}/")))
}

fn canonical_url_for_podcast(fm: &PodcastFrontMatter, app_config: &AppConfig) -> String {
    // Use the frontmatter slug, NOT the filename stem.
    app_config.app_url(&format!("/podcast/{}", fm.slug))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> AppConfig {
        AppConfig {
            base_url: url::Url::parse("https://coreyja.com").unwrap(),
            imgproxy_url: None,
        }
    }

    fn write_file(dir: &Path, name: &str, body: &str) -> PathBuf {
        let p = dir.join(name);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).expect("mkdir");
        }
        std::fs::write(&p, body).expect("write");
        p
    }

    // ==================== Classifiers ====================

    #[test]
    fn classify_blog_skips_already_published() {
        let tmp = tempfile::tempdir().unwrap();
        let p = write_file(
            tmp.path(),
            "index.md",
            "---\ntitle: T\ndate: 2026-05-25\nlinkedin_url: https://x\n---\n\nbody\n",
        );
        assert!(classify_blog(&p).unwrap().is_none());
    }

    #[test]
    fn classify_blog_skips_before_cutoff() {
        let tmp = tempfile::tempdir().unwrap();
        let p = write_file(
            tmp.path(),
            "index.md",
            "---\ntitle: T\ndate: 2024-01-01\n---\n\nbody\n",
        );
        assert!(classify_blog(&p).unwrap().is_none());
    }

    #[test]
    fn classify_blog_skips_newsletters() {
        let tmp = tempfile::tempdir().unwrap();
        let p = write_file(
            tmp.path(),
            "index.md",
            "---\ntitle: T\ndate: 2026-05-25\nis_newsletter: true\n---\n\nbody\n",
        );
        assert!(classify_blog(&p).unwrap().is_none());
    }

    #[test]
    fn classify_blog_accepts_cutoff_day_boundary() {
        let tmp = tempfile::tempdir().unwrap();
        let p = write_file(
            tmp.path(),
            "index.md",
            "---\ntitle: T\ndate: 2026-05-23\n---\n\nbody\n",
        );
        assert!(classify_blog(&p).unwrap().is_some());
    }

    #[test]
    fn classify_newsletter_requires_is_newsletter_flag() {
        let tmp = tempfile::tempdir().unwrap();
        let p = write_file(
            tmp.path(),
            "index.md",
            "---\ntitle: T\ndate: 2026-05-25\n---\n\nbody\n",
        );
        assert!(classify_newsletter(&p).unwrap().is_none());
    }

    #[test]
    fn classify_podcast_accepts_eligible() {
        let tmp = tempfile::tempdir().unwrap();
        let p = write_file(
            tmp.path(),
            "ep5.md",
            "---\ntitle: T\ndate: 2026-05-25\nslug: ep5\nyoutube_id: abc\naudio_url: https://x/audio.mp3\naudio_length_bytes: 1\naudio_duration: \"01:00:00\"\n---\n\nbody\n",
        );
        assert!(classify_podcast(&p).unwrap().is_some());
    }

    // ==================== update_frontmatter_with_linkedin_url ====================

    #[test]
    fn update_frontmatter_with_linkedin_url_basic() {
        let content = "---\ntitle: T\ndate: 2026-05-25\n---\n\nBody.\n";
        let updated = update_frontmatter_with_linkedin_url(content, "https://x").unwrap();
        assert!(updated.contains("linkedin_url: https://x"));
        assert!(updated.contains("Body."));
    }

    #[test]
    fn update_frontmatter_with_linkedin_url_returns_err_on_malformed() {
        let bad = "---\ntitle: T\ndate: 2026-05-25\n\nNo closing.\n";
        assert!(update_frontmatter_with_linkedin_url(bad, "https://x").is_err());
    }

    #[test]
    fn update_frontmatter_with_linkedin_url_returns_err_if_already_present() {
        let content =
            "---\ntitle: T\ndate: 2026-05-25\nlinkedin_url: https://existing\n---\n\nBody.\n";
        assert!(update_frontmatter_with_linkedin_url(content, "https://new").is_err());
    }

    #[test]
    fn update_frontmatter_preserves_other_syndication_urls() {
        let content =
            "---\ntitle: T\ndate: 2026-05-25\nbsky_url: https://bsky.app/x\n---\n\nBody.\n";
        let updated = update_frontmatter_with_linkedin_url(content, "https://li").unwrap();
        assert!(updated.contains("bsky_url: https://bsky.app/x"));
        assert!(updated.contains("linkedin_url: https://li"));
    }

    // ==================== Canonical URLs ====================

    #[test]
    fn canonical_url_for_blog_basic() {
        let cfg = test_config();
        let p = PathBuf::from("blog/foo/index.md");
        assert_eq!(
            canonical_url_for_blog(&p, &cfg).unwrap(),
            "https://coreyja.com/posts/foo/"
        );
    }

    #[test]
    fn canonical_url_for_newsletter_basic() {
        let cfg = test_config();
        let p = PathBuf::from("blog/weekly/20260601/index.md");
        assert_eq!(
            canonical_url_for_newsletter(&p, &cfg).unwrap(),
            "https://coreyja.com/posts/weekly/20260601/"
        );
    }

    #[test]
    fn canonical_url_for_podcast_uses_slug_not_filename() {
        let cfg = test_config();
        let fm = PodcastFrontMatter {
            title: "T".into(),
            date: NaiveDate::from_ymd_opt(2026, 5, 25).unwrap(),
            slug: "ep5".into(),
            youtube_id: "abc".into(),
            youtube_url: None,
            audio_url: "https://x/audio.mp3".into(),
            audio_length_bytes: 1,
            audio_duration: "01:00:00".into(),
            transcript_url: None,
            linkedin_url: None,
            linkedin_content: None,
        };
        assert_eq!(
            canonical_url_for_podcast(&fm, &cfg),
            "https://coreyja.com/podcast/ep5"
        );
    }

    // ==================== Sibling paths ====================

    #[test]
    fn sibling_linkedin_path_for_blog() {
        let p = PathBuf::from("blog/foo/index.md");
        assert_eq!(
            sibling_linkedin_path_for(&p),
            PathBuf::from("blog/foo/linkedin.md")
        );
    }

    #[test]
    fn sibling_linkedin_path_for_podcast() {
        let p = PathBuf::from("podcast/ep5.md");
        assert_eq!(
            sibling_linkedin_path_for(&p),
            PathBuf::from("podcast/ep5.linkedin.md")
        );
    }

    // ==================== Body source ordering ====================

    #[test]
    fn body_source_prefers_sibling_file_over_frontmatter() {
        let tmp = tempfile::tempdir().unwrap();
        let post = write_file(tmp.path(), "index.md", "stub");
        let _sibling = write_file(tmp.path(), "linkedin.md", "From sibling file.\n");
        let out = read_custom_body(&post, Some("from frontmatter")).unwrap();
        assert_eq!(out.trim(), "From sibling file.");
    }

    #[test]
    fn body_source_falls_back_to_first_paragraph_when_sibling_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let post = write_file(tmp.path(), "index.md", "stub");
        let _sibling = write_file(tmp.path(), "linkedin.md", "   \n");
        let out = read_custom_body(&post, None);
        assert!(out.is_none());
    }

    #[test]
    fn body_source_skips_empty_linkedin_content_field() {
        let tmp = tempfile::tempdir().unwrap();
        let post = write_file(tmp.path(), "index.md", "stub");
        let out = read_custom_body(&post, Some("   "));
        assert!(out.is_none());
    }
}
