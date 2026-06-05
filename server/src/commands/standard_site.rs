use std::path::{Path, PathBuf};
use std::str::FromStr;

use cja::color_eyre::eyre::eyre;
use clap::Subcommand;
use serde::{Deserialize, Serialize};

use crate::bluesky::{
    at_uri_to_web_url, Blob, BlueskyClient, BlueskyConfig, DocumentRecord, PublicationRecord,
    StrongRef,
};
use crate::commands::frontmatter;
use posts::blog::{BlogFrontMatter, ToCanonicalPath};
use posts::plain::IntoPlainText;
use posts::MarkdownAst;

/// Posts dated on or after this cutoff get a Bluesky post in addition to
/// their `site.standard.document` record. Older posts are syndicated to
/// standard.site only — preserves the historical Bluesky feed instead of
/// flooding it with backfill posts on the first deploy.
const BSKY_POST_CUTOFF_DATE: &str = "2026-05-29";

fn bsky_post_cutoff() -> chrono::NaiveDate {
    chrono::NaiveDate::parse_from_str(BSKY_POST_CUTOFF_DATE, "%Y-%m-%d")
        .expect("BSKY_POST_CUTOFF_DATE valid")
}

#[derive(Subcommand, Debug)]
pub enum StandardSiteCommand {
    /// Create the `site.standard.publication` record on the PDS and cache
    /// its `at_uri` + `at_cid` back into `publications.toml`. Idempotent: if
    /// `at_uri` is already set, refresh the record via `putRecord` using the
    /// rkey parsed out of the cached `at_uri`.
    Init(InitArgs),
    /// Walk the publication's `content_dir`, create/update a
    /// `site.standard.document` for every post, then create the bsky post
    /// with `associatedRefs`.
    Sync(SyncArgs),
}

#[derive(clap::Args, Debug)]
pub struct InitArgs {
    pub key: String,
    /// Override the default config path. The config's parent dir is the repo root.
    #[arg(long, default_value = "publications.toml")]
    pub config: PathBuf,
    /// Re-upload the publication record even if `at_uri`/`at_cid` are already
    /// cached. Without this, init is a no-op when the publication is already
    /// bootstrapped — letting the workflow run it on every deploy.
    #[arg(long)]
    pub force: bool,
}

#[derive(clap::Args, Debug)]
#[group(required = true, multiple = false)]
pub struct SyncArgs {
    #[arg(long, group = "target")]
    pub all: bool,
    #[arg(long, group = "target")]
    pub key: Option<String>,
    #[arg(long, default_value = "publications.toml")]
    pub config: PathBuf,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PublicationsConfig {
    #[serde(rename = "publication")]
    pub publications: Vec<PublicationConfig>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PublicationConfig {
    pub key: String,
    pub title: String,
    pub description: String,
    pub url: String,
    pub content_dir: String,
    pub collection: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub at_cid: Option<String>,
    /// `true` once the publication's branded OG cover blob has been
    /// successfully uploaded and attached to the publication record. The
    /// init step short-circuits when this is `true` AND `at_uri`/`at_cid`
    /// are cached — so deploys don't churn the publication record.
    ///
    /// Flip back to `false` (or remove the line) to trigger an in-place
    /// refresh on the next deploy: init re-puts the publication with a
    /// new cid, sync detects the per-document `atproto_pub_cid` drift,
    /// and every document gets re-put with the refreshed strong-ref.
    #[serde(default)]
    pub cover_synced: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SyncOutcome {
    /// Both `atproto_uri` and `bsky_url` already set — nothing to do.
    Skip,
    /// Document already synced but no bsky post yet.
    BskyOnly,
    /// Neither side synced — create document and bsky post.
    Both,
}

fn classify_blog_post(fm: &BlogFrontMatter) -> SyncOutcome {
    match (&fm.atproto_uri, &fm.bsky_url) {
        (Some(_), Some(_)) => SyncOutcome::Skip,
        (Some(_), None) => SyncOutcome::BskyOnly,
        _ => SyncOutcome::Both,
    }
}

fn load_config(path: &Path) -> cja::Result<PublicationsConfig> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| eyre!("Failed to read config {}: {}", path.display(), e))?;
    toml::from_str(&raw).map_err(|e| eyre!("Failed to parse {}: {}", path.display(), e))
}

fn save_config(path: &Path, cfg: &PublicationsConfig) -> cja::Result<()> {
    let serialized =
        toml::to_string_pretty(cfg).map_err(|e| eyre!("Failed to serialize config: {}", e))?;
    std::fs::write(path, serialized)
        .map_err(|e| eyre!("Failed to write {}: {}", path.display(), e))?;
    Ok(())
}

fn repo_root_for(config_path: &Path) -> PathBuf {
    config_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn rkey_from_at_uri(at_uri: &str) -> cja::Result<String> {
    let stripped = at_uri
        .strip_prefix("at://")
        .ok_or_else(|| eyre!("Invalid AT URI: missing at:// prefix: {at_uri}"))?;
    let (_did, rest) = stripped
        .split_once('/')
        .ok_or_else(|| eyre!("Invalid AT URI: missing collection: {at_uri}"))?;
    let (_collection, rkey) = rest
        .split_once('/')
        .ok_or_else(|| eyre!("Invalid AT URI: missing record key: {at_uri}"))?;
    if rkey.is_empty() {
        return Err(eyre!("Invalid AT URI: empty rkey: {at_uri}"));
    }
    Ok(rkey.to_string())
}

fn rkey_from_blog_path(relative_path: &Path) -> String {
    // Each blog post lives at `<slug>/index.md`. Strip `index.md` and use
    // the parent directory path as the rkey, replacing `/` with `-` so
    // nested paths (`weekly/20230713/index.md`) collapse to a single segment.
    let parent = relative_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    if parent.as_os_str().is_empty() {
        return relative_path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
    }
    parent
        .to_string_lossy()
        .replace([std::path::MAIN_SEPARATOR, '/'], "-")
}

fn relative_under(root: &Path, content_dir: &str, post_path: &Path) -> cja::Result<PathBuf> {
    let prefix = root.join(content_dir);
    post_path
        .strip_prefix(&prefix)
        .map(Path::to_path_buf)
        .map_err(|_| {
            eyre!(
                "Post path {} is not under {}",
                post_path.display(),
                prefix.display()
            )
        })
}

/// Recursively walk `root` collecting any file named exactly `index.md`.
fn collect_index_md_files(root: &Path) -> cja::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    walk_index_md(root, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk_index_md(dir: &Path, out: &mut Vec<PathBuf>) -> cja::Result<()> {
    let entries =
        std::fs::read_dir(dir).map_err(|e| eyre!("Failed to read dir {}: {}", dir.display(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| eyre!("Failed to read dir entry: {}", e))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| eyre!("Failed to stat {}: {}", path.display(), e))?;
        if file_type.is_dir() {
            walk_index_md(&path, out)?;
        } else if file_type.is_file()
            && path.file_name().and_then(|s| s.to_str()) == Some("index.md")
        {
            out.push(path);
        }
    }
    Ok(())
}

/// Build the imgproxy URL that rasterizes the publication's SVG OG card to a
/// 1200×630 PNG. Mirrors the format used by
/// `templates::og::og_image_url` so cover images match the per-post cards.
fn publication_cover_imgproxy_url(app_base_url: &str, imgproxy_url: &str, key: &str) -> String {
    let svg_url = format!(
        "{}/og/publication/{}.svg",
        app_base_url.trim_end_matches('/'),
        key
    );
    format!(
        "{}/unsafe/rs:fill:1200:630/format:png/plain/{}",
        imgproxy_url.trim_end_matches('/'),
        urlencoding::encode(&svg_url)
    )
}

/// Read a required env var, treating both "unset" and "set-to-empty-string"
/// as missing. Without the empty-string check, an unset CI secret like
/// `IMGPROXY_URL: ${{ secrets.IMGPROXY_URL }}` produces an empty string
/// rather than a missing var, which silently feeds an empty base into URL
/// construction and yields a malformed relative URL at fetch time.
fn required_env(name: &str) -> cja::Result<String> {
    match std::env::var(name) {
        Ok(v) if !v.is_empty() => Ok(v),
        _ => Err(eyre!("{name} must be set (and non-empty)")),
    }
}

/// Fetch the publication's branded OG card as a PNG via imgproxy.
///
/// Requires `APP_BASE_URL` (so we know where the deployed SVG endpoint lives)
/// and `IMGPROXY_URL` (so the SVG gets rasterized). Returns `(bytes, mime)`
/// suitable for `upload_blob`.
async fn fetch_publication_cover_png(key: &str) -> cja::Result<(Vec<u8>, String)> {
    let app_base_url = required_env("APP_BASE_URL")?;
    let imgproxy_url = required_env("IMGPROXY_URL")?;
    let png_url = publication_cover_imgproxy_url(&app_base_url, &imgproxy_url, key);

    let resp = reqwest::get(&png_url)
        .await
        .map_err(|e| eyre!("Failed to fetch cover from {png_url}: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(eyre!("imgproxy returned {status} for {png_url}"));
    }
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/png")
        .to_string();
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| eyre!("Failed to read cover body: {e}"))?
        .to_vec();
    Ok((bytes, content_type))
}

pub async fn run(cmd: &StandardSiteCommand) -> cja::Result<()> {
    let config = BlueskyConfig::from_env()?;
    let client = BlueskyClient::login(&config).await?;

    match cmd {
        StandardSiteCommand::Init(args) => {
            init_publication(&client, &args.config, &args.key, args.force).await
        }
        StandardSiteCommand::Sync(args) => {
            let cfg = load_config(&args.config)?;
            let repo_root = repo_root_for(&args.config);
            let to_sync: Vec<PublicationConfig> = if args.all {
                cfg.publications.clone()
            } else {
                let key = args.key.as_ref().ok_or_else(|| {
                    eyre!("Either --all or --key must be provided to publish-standard-site sync")
                })?;
                cfg.publications
                    .iter()
                    .find(|p| &p.key == key)
                    .cloned()
                    .map(|p| vec![p])
                    .ok_or_else(|| eyre!("Publication '{}' not found in config", key))?
            };

            let mut total_failed = 0usize;
            for pub_cfg in &to_sync {
                let summary = sync_publication(&client, pub_cfg, &repo_root).await?;
                println!(
                    "Publication '{}': {} synced, {} skipped, {} failed",
                    pub_cfg.key, summary.synced, summary.skipped, summary.failed
                );
                total_failed += summary.failed;
            }

            if total_failed > 0 {
                return Err(eyre!(
                    "{total_failed} post(s) failed to sync; next run will retry"
                ));
            }
            Ok(())
        }
    }
}

#[derive(Debug, Default)]
struct SyncSummary {
    synced: usize,
    skipped: usize,
    failed: usize,
}

async fn init_publication(
    client: &BlueskyClient,
    config_path: &Path,
    key: &str,
    force: bool,
) -> cja::Result<()> {
    let mut cfg = load_config(config_path)?;
    let pub_cfg = cfg
        .publications
        .iter_mut()
        .find(|p| p.key == key)
        .ok_or_else(|| eyre!("Publication '{key}' not found in {}", config_path.display()))?;

    // Idempotent fast path: when the publication is cached AND its cover
    // has been confirmed uploaded, do nothing. Without the `cover_synced`
    // gate we'd skip a repair-and-recover case (publication was created in
    // a previous deploy where cover fetch failed; `at_uri`/`at_cid` are
    // populated but the publication record on the PDS has `cover: None`).
    if !force && pub_cfg.at_uri.is_some() && pub_cfg.at_cid.is_some() && pub_cfg.cover_synced {
        println!(
            "Publication '{key}' already initialized with cover (uri={}); skipping.",
            pub_cfg.at_uri.as_deref().unwrap_or("")
        );
        return Ok(());
    }

    // Fetch the publication's branded OG card as a PNG via imgproxy. The SVG
    // itself is served by the deployed app at `/og/publication/{key}.svg`;
    // imgproxy rasterizes and caches the 1200×630 PNG that the cover blob
    // points at. Best-effort: if the fetch fails we proceed without a cover
    // — `cover_synced` stays `false` so the next deploy retries.
    let (cover, cover_synced): (Option<Blob>, bool) =
        match fetch_publication_cover_png(&pub_cfg.key).await {
            Ok((bytes, mime)) => match client.upload_blob(bytes, &mime).await {
                Ok(blob) => (Some(blob), true),
                Err(e) => {
                    eprintln!(
                        "Warning: cover upload failed for '{}': {e}. Proceeding without cover.",
                        pub_cfg.key
                    );
                    (None, false)
                }
            },
            Err(e) => {
                eprintln!(
                "Warning: could not fetch generated cover for '{}': {e}. Proceeding without cover.",
                pub_cfg.key
            );
                (None, false)
            }
        };

    let record = PublicationRecord {
        record_type: "site.standard.publication".to_string(),
        name: pub_cfg.title.clone(),
        description: Some(pub_cfg.description.clone()),
        url: pub_cfg.url.clone(),
        icon: cover,
    };

    let response = if pub_cfg.at_uri.is_none() || pub_cfg.at_cid.is_none() {
        client.create_publication(record).await?
    } else {
        let cached = pub_cfg.at_uri.as_deref().unwrap();
        let rkey = rkey_from_at_uri(cached)?;
        client.put_publication(&rkey, record).await?
    };

    pub_cfg.at_uri = Some(response.uri.clone());
    pub_cfg.at_cid = Some(response.cid.clone());
    pub_cfg.cover_synced = cover_synced;
    save_config(config_path, &cfg)?;

    println!(
        "Publication '{key}' synced: uri={} cid={} cover_synced={cover_synced}",
        response.uri, response.cid
    );
    Ok(())
}

async fn sync_publication(
    client: &BlueskyClient,
    pub_cfg: &PublicationConfig,
    repo_root: &Path,
) -> cja::Result<SyncSummary> {
    match (&pub_cfg.at_uri, &pub_cfg.at_cid) {
        (Some(_), Some(_)) => {}
        (Some(_), None) => {
            return Err(eyre!(
                "publication '{}' is partially bootstrapped — re-run `publish-standard-site init {}`",
                pub_cfg.key,
                pub_cfg.key
            ));
        }
        _ => {
            return Err(eyre!(
                "publication '{}' is not bootstrapped — run `publish-standard-site init {}` first",
                pub_cfg.key,
                pub_cfg.key
            ));
        }
    }

    let content_root = repo_root.join(&pub_cfg.content_dir);
    let post_paths = collect_index_md_files(&content_root)?;

    let mut summary = SyncSummary::default();
    let mut failures: Vec<(PathBuf, cja::color_eyre::Report)> = Vec::new();

    for post_path in &post_paths {
        match sync_one(client, pub_cfg, post_path, repo_root).await {
            Ok(SyncOutcome::Skip) => summary.skipped += 1,
            Ok(_) => summary.synced += 1,
            Err(e) => {
                eprintln!("Failed to sync {}: {e}", post_path.display());
                failures.push((post_path.clone(), e));
            }
        }
    }

    summary.failed = failures.len();
    Ok(summary)
}

async fn sync_one(
    client: &BlueskyClient,
    pub_cfg: &PublicationConfig,
    post_path: &Path,
    repo_root: &Path,
) -> cja::Result<SyncOutcome> {
    let content = std::fs::read_to_string(post_path)
        .map_err(|e| eyre!("Failed to read {}: {}", post_path.display(), e))?;
    let (yaml, _body) = frontmatter::split_frontmatter(&content)?;
    let fm: BlogFrontMatter =
        serde_yaml::from_str(yaml).map_err(|e| eyre!("Invalid frontmatter: {e}"))?;

    // Filter by `publication` field — only sync posts that belong to this publication.
    if fm.publication != pub_cfg.key {
        return Ok(SyncOutcome::Skip);
    }

    let pub_cid = pub_cfg
        .at_cid
        .as_deref()
        .ok_or_else(|| eyre!("publication at_cid missing"))?;
    let pub_uri = pub_cfg
        .at_uri
        .as_deref()
        .ok_or_else(|| eyre!("publication at_uri missing"))?;
    let pub_ref = StrongRef {
        uri: pub_uri.to_string(),
        cid: pub_cid.to_string(),
    };

    let is_historical = fm.date < bsky_post_cutoff();
    let doc_exists = fm.atproto_uri.is_some();
    let doc_pinned_to_current_pub = fm.atproto_pub_cid.as_deref() == Some(pub_cid);
    let bsky_exists = fm.bsky_url.is_some();

    // What needs doing for this post?
    // - Document put: any time the doc doesn't exist OR its pinned pub cid has drifted.
    // - Bsky post: only when the post is recent (>= cutoff) and we don't have a bsky_url.
    //   When we DO need a bsky post but the doc is already current, we still re-put the
    //   doc so we have a fresh strong-ref to attach to the bsky post.
    let need_bsky_post = !bsky_exists && !is_historical;
    let need_doc_put = !doc_exists || !doc_pinned_to_current_pub || need_bsky_post;

    if !need_doc_put && !need_bsky_post {
        return Ok(SyncOutcome::Skip);
    }

    let rel = relative_under(repo_root, &pub_cfg.content_dir, post_path)?;
    let blog_rkey = rkey_from_blog_path(&rel);
    let canonical = rel.canonical_path();
    let post_url = format!("https://coreyja.com/posts/{canonical}");
    // `path` per spec is the URL path segment relative to the publication URL.
    // Publication URL is `https://coreyja.com/posts`; the per-post URL is
    // `https://coreyja.com/posts/<canonical>`. So path is `/<canonical>`.
    let path = format!("/{canonical}");

    let ast = MarkdownAst::from_str(&content)?;
    let description: String = ast.0.plain_text().chars().take(100).collect();

    // Re-put the document with the current publication URI. Idempotent on
    // rkey; produces a fresh doc cid we attach to the bsky post if needed.
    let record = build_document_record(&fm, pub_uri, path, description.clone());
    let doc_response = client.put_document(&blog_rkey, record).await?;
    let doc_ref = StrongRef {
        uri: doc_response.uri.clone(),
        cid: doc_response.cid.clone(),
    };

    // Conditionally publish a Bluesky post (only for recent posts without one).
    let bsky_web_url_opt = if need_bsky_post {
        let bsky_response = client
            .create_blog_post(
                &fm.title,
                &post_url,
                &description,
                vec![pub_ref.clone(), doc_ref.clone()],
            )
            .await?;
        Some(at_uri_to_web_url(&bsky_response.uri)?)
    } else {
        None
    };

    // Frontmatter updates: atproto_uri (if first put), atproto_pub_cid (every
    // doc put), bsky_url (if we just created a bsky post). We always write
    // atproto_pub_cid since the doc was just put against the current pub_cid.
    let mut new_keys: Vec<(&str, &str)> = Vec::new();
    if !doc_exists {
        new_keys.push(("atproto_uri", &doc_ref.uri));
    }
    new_keys.push(("atproto_pub_cid", pub_cid));
    let bsky_web_url = bsky_web_url_opt.unwrap_or_default();
    if !bsky_web_url.is_empty() {
        new_keys.push(("bsky_url", &bsky_web_url));
    }
    let updated = frontmatter::append_frontmatter_keys(&content, &new_keys);
    write_back(post_path, &updated)?;

    // Outcome reporting: distinguish between "both sides changed" and
    // "only bsky changed" for the summary printout. A doc-only re-put
    // (drift refresh) reports as Both since we did write doc-side state.
    if need_bsky_post && (!doc_exists || !doc_pinned_to_current_pub) {
        Ok(SyncOutcome::Both)
    } else if need_bsky_post {
        Ok(SyncOutcome::BskyOnly)
    } else {
        Ok(SyncOutcome::Both)
    }
}

fn build_document_record(
    fm: &BlogFrontMatter,
    site: &str,
    path: String,
    description: String,
) -> DocumentRecord {
    let published_at = fm
        .date
        .and_hms_opt(0, 0, 0)
        .expect("midnight valid")
        .and_utc()
        .to_rfc3339();
    DocumentRecord {
        record_type: "site.standard.document".to_string(),
        site: site.to_string(),
        title: fm.title.clone(),
        published_at,
        path: Some(path),
        description: Some(description),
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
        tags: fm.tags.clone(),
        cover_image: None,
    }
}

fn write_back(post_path: &Path, content: &str) -> cja::Result<()> {
    std::fs::write(post_path, content)
        .map_err(|e| eyre!("Failed to write {}: {}", post_path.display(), e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use std::io::Write;

    fn sample_publication() -> PublicationConfig {
        PublicationConfig {
            key: "blog".to_string(),
            title: "coreyja".to_string(),
            description: "Personal blog".to_string(),
            url: "https://coreyja.com/posts".to_string(),
            content_dir: "blog".to_string(),
            collection: "site.standard.document".to_string(),
            at_uri: None,
            at_cid: None,
            cover_synced: false,
        }
    }

    fn sample_blog_fm() -> BlogFrontMatter {
        BlogFrontMatter {
            title: "T".to_string(),
            date: NaiveDate::default(),
            is_newsletter: false,
            bsky_url: None,
            newsletter_send_at: None,
            buttondown_id: None,
            og_image: None,
            subtitle: None,
            tags: vec![],
            author: None,
            atproto_uri: None,
            atproto_pub_cid: None,
            publication: "blog".to_string(),
        }
    }

    #[test]
    fn load_config_parses_blog_entry() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("publications.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(
            f,
            "[[publication]]\nkey = \"blog\"\ntitle = \"coreyja\"\ndescription = \"d\"\nurl = \"https://coreyja.com/posts\"\ncontent_dir = \"blog\"\ncollection = \"site.standard.document\""
        )
        .unwrap();
        let cfg = load_config(&path).unwrap();
        assert_eq!(cfg.publications.len(), 1);
        let p = &cfg.publications[0];
        assert_eq!(p.key, "blog");
        assert_eq!(p.title, "coreyja");
        assert_eq!(p.content_dir, "blog");
        assert!(p.at_uri.is_none());
        assert!(p.at_cid.is_none());
    }

    #[test]
    fn publication_cover_imgproxy_url_format() {
        let out = publication_cover_imgproxy_url(
            "https://coreyja.com",
            "https://img.coreyja.com",
            "blog",
        );
        assert!(
            out.starts_with("https://img.coreyja.com/unsafe/rs:fill:1200:630/format:png/plain/")
        );
        assert!(out.ends_with(
            &urlencoding::encode("https://coreyja.com/og/publication/blog.svg").into_owned()
        ));
    }

    #[test]
    fn publication_cover_imgproxy_url_strips_trailing_slashes() {
        let out = publication_cover_imgproxy_url(
            "https://coreyja.com/",
            "https://img.coreyja.com/",
            "blog",
        );
        assert!(!out.contains("com//og/"));
        assert!(!out.contains("com//unsafe/"));
    }

    #[test]
    fn save_config_roundtrips() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("publications.toml");
        let cfg = PublicationsConfig {
            publications: vec![sample_publication()],
        };
        save_config(&path, &cfg).unwrap();
        let loaded = load_config(&path).unwrap();
        assert_eq!(loaded.publications.len(), 1);
        assert_eq!(loaded.publications[0].key, "blog");
        assert_eq!(loaded.publications[0].title, "coreyja");
    }

    #[test]
    fn save_config_preserves_at_uri_and_cid_after_init() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("publications.toml");
        let mut pubc = sample_publication();
        pubc.at_uri = Some("at://did:plc:abc/site.standard.publication/3xyz".to_string());
        pubc.at_cid = Some("bafy123".to_string());
        let cfg = PublicationsConfig {
            publications: vec![pubc],
        };
        save_config(&path, &cfg).unwrap();
        let loaded = load_config(&path).unwrap();
        assert_eq!(
            loaded.publications[0].at_uri.as_deref(),
            Some("at://did:plc:abc/site.standard.publication/3xyz")
        );
        assert_eq!(loaded.publications[0].at_cid.as_deref(), Some("bafy123"));
    }

    #[test]
    fn classify_blog_post_skip_when_both_set() {
        let mut fm = sample_blog_fm();
        fm.atproto_uri = Some("at://abc".to_string());
        fm.bsky_url = Some("https://bsky.app/x".to_string());
        assert_eq!(classify_blog_post(&fm), SyncOutcome::Skip);
    }

    #[test]
    fn classify_blog_post_bsky_only_when_doc_set() {
        let mut fm = sample_blog_fm();
        fm.atproto_uri = Some("at://abc".to_string());
        fm.bsky_url = None;
        assert_eq!(classify_blog_post(&fm), SyncOutcome::BskyOnly);
    }

    #[test]
    fn classify_blog_post_both_when_neither_set() {
        let fm = sample_blog_fm();
        assert_eq!(classify_blog_post(&fm), SyncOutcome::Both);
    }

    #[test]
    fn rkey_from_blog_path_strips_index_md() {
        let p = Path::new("look-ma-no-ai/index.md");
        assert_eq!(rkey_from_blog_path(p), "look-ma-no-ai");
    }

    #[test]
    fn rkey_from_blog_path_handles_newsletter_paths() {
        let p = Path::new("weekly/20230713/index.md");
        assert_eq!(rkey_from_blog_path(p), "weekly-20230713");
    }

    #[test]
    fn rkey_from_at_uri_parses_collection_and_rkey() {
        let rkey = rkey_from_at_uri("at://did:plc:abc/site.standard.publication/3labc").unwrap();
        assert_eq!(rkey, "3labc");
    }

    #[test]
    fn rkey_from_at_uri_rejects_malformed_uri() {
        assert!(rkey_from_at_uri("not-an-at-uri").is_err());
        assert!(rkey_from_at_uri("at://did:plc:abc").is_err());
        assert!(rkey_from_at_uri("at://did:plc:abc/collection").is_err());
    }

    #[test]
    fn repo_root_for_default_path_returns_dot() {
        assert_eq!(
            repo_root_for(Path::new("publications.toml")),
            PathBuf::from(".")
        );
    }

    #[test]
    fn repo_root_for_nested_path_returns_parent() {
        assert_eq!(
            repo_root_for(Path::new("/tmp/repo/publications.toml")),
            PathBuf::from("/tmp/repo")
        );
    }

    #[test]
    fn relative_under_strips_repo_and_content_dir() {
        let rel = relative_under(
            Path::new("/r"),
            "blog",
            Path::new("/r/blog/look-ma-no-ai/index.md"),
        )
        .unwrap();
        assert_eq!(rel, PathBuf::from("look-ma-no-ai/index.md"));
    }

    /// Walks `blog/**/index.md` to ensure every publishable post fits within
    /// Bluesky's 300-character post limit (title + url + separators only,
    /// since the body is replaced by the standard.site card). Historical
    /// posts are exempt — they syndicate to standard.site only, no bsky post.
    #[test]
    fn all_publishable_blog_posts_fit_within_bsky_post_limit() {
        let blog_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("server has a parent dir")
            .join("blog");
        let cutoff = bsky_post_cutoff();

        let posts = collect_index_md_files(&blog_dir).expect("walk blog/");
        let mut failures = Vec::new();

        for path in &posts {
            let Ok(content) = std::fs::read_to_string(path) else {
                continue;
            };
            let Ok((yaml, _body)) = frontmatter::split_frontmatter(&content) else {
                continue;
            };
            let Ok(fm): Result<BlogFrontMatter, _> = serde_yaml::from_str(yaml) else {
                continue;
            };
            if fm.date < cutoff {
                continue;
            }
            let rel = path.strip_prefix(&blog_dir).unwrap_or(path);
            let canonical = rel.to_path_buf().canonical_path();
            let url = format!("https://coreyja.com/posts/{canonical}");
            let chars = fm.title.chars().count() + 2 + url.chars().count();
            if chars > 300 {
                failures.push(format!(
                    "{}: {chars} chars (over by {})",
                    path.display(),
                    chars - 300
                ));
            }
        }

        assert!(
            failures.is_empty(),
            "These blog posts would exceed Bluesky's 300-character post limit:\n  {}",
            failures.join("\n  ")
        );
    }

    /// Every blog post must produce a unique rkey (since the document record
    /// is keyed by it). Duplicates would silently overwrite each other.
    #[test]
    fn all_blog_posts_have_unique_rkeys() {
        let blog_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("server has a parent dir")
            .join("blog");
        let posts = collect_index_md_files(&blog_dir).expect("walk blog/");
        let mut seen: std::collections::HashMap<String, PathBuf> = std::collections::HashMap::new();
        let mut conflicts = Vec::new();
        for path in &posts {
            let rel = path.strip_prefix(&blog_dir).unwrap_or(path).to_path_buf();
            let rkey = rkey_from_blog_path(&rel);
            if let Some(existing) = seen.insert(rkey.clone(), path.clone()) {
                conflicts.push(format!(
                    "rkey '{rkey}' shared by {} and {}",
                    existing.display(),
                    path.display()
                ));
            }
        }
        assert!(
            conflicts.is_empty(),
            "Duplicate rkeys would overwrite each other on the PDS:\n  {}",
            conflicts.join("\n  ")
        );
    }
}
