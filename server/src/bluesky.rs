use serde::{Deserialize, Serialize};

pub struct BlueskyConfig {
    pub identifier: String,
    pub app_password: String,
}

impl BlueskyConfig {
    pub fn from_env() -> cja::Result<Self> {
        let identifier = std::env::var("BLUESKY_IDENTIFIER").map_err(|_| {
            cja::color_eyre::eyre::eyre!("Missing BLUESKY_IDENTIFIER environment variable")
        })?;
        let app_password = std::env::var("BLUESKY_APP_PASSWORD").map_err(|_| {
            cja::color_eyre::eyre::eyre!("Missing BLUESKY_APP_PASSWORD environment variable")
        })?;
        Ok(Self {
            identifier,
            app_password,
        })
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SessionResponse {
    did: String,
    access_jwt: String,
}

pub struct BlueskyClient {
    client: reqwest::Client,
    session: SessionResponse,
    /// PDS base URL discovered from the user's DID document. All XRPC calls
    /// after login go here, not to a hardcoded host — supports accounts on
    /// any PDS (Blacksky, Tranquil, self-hosted, bsky.social, etc.).
    pds_url: String,
}

/// Slingshot is an `ATProto` identity resolver run by Microcosm
/// (<https://slingshot.microcosm.blue>). One call returns DID, handle, and
/// PDS for any identifier — handle or DID, did:plc or did:web. Replaces
/// the multi-step PLC directory / .well-known resolution dance.
///
/// The endpoint is in the experimental `com.bad-example.*` namespace. If
/// the NSID is ever renamed, that's a one-line change here.
const SLINGSHOT_RESOLVE_URL: &str =
    "https://slingshot.microcosm.blue/xrpc/com.bad-example.identity.resolveMiniDoc";

#[derive(Deserialize, Debug)]
struct MiniDoc {
    #[allow(dead_code)] // returned by API; kept for future use
    did: String,
    #[allow(dead_code)] // returned by API; kept for future use
    handle: Option<String>,
    pds: String,
}

/// Resolve any identifier (handle or DID) to its PDS endpoint via Slingshot.
async fn resolve_pds(client: &reqwest::Client, identifier: &str) -> cja::Result<String> {
    let url = format!("{SLINGSHOT_RESOLVE_URL}?identifier={identifier}");
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(cja::color_eyre::eyre::eyre!(
            "Slingshot resolveMiniDoc failed for '{}' ({}): {}",
            identifier,
            resp.status(),
            resp.text().await.unwrap_or_default()
        ));
    }
    let doc: MiniDoc = resp.json().await?;
    Ok(doc.pds.trim_end_matches('/').to_string())
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CreateSessionRequest {
    identifier: String,
    password: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CreateRecordRequest {
    repo: String,
    collection: String,
    record: PostRecord,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PostRecord {
    #[serde(rename = "$type")]
    record_type: String,
    text: String,
    created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    facets: Option<Vec<Facet>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embed: Option<EmbedExternal>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Facet {
    index: ByteSlice,
    features: Vec<FacetFeature>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ByteSlice {
    byte_start: usize,
    byte_end: usize,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct FacetFeature {
    #[serde(rename = "$type")]
    feature_type: String,
    uri: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct EmbedExternal {
    #[serde(rename = "$type")]
    embed_type: String,
    external: ExternalEmbed,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ExternalEmbed {
    uri: String,
    title: String,
    description: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRecordResponse {
    pub uri: String,
}

impl BlueskyClient {
    pub async fn login(config: &BlueskyConfig) -> cja::Result<Self> {
        let client = reqwest::Client::new();

        // Resolve identifier -> PDS so we authenticate against the account's
        // actual host, not a hardcoded one. Works for any account on any PDS
        // in the ATProto network.
        let pds_url = resolve_pds(&client, &config.identifier).await?;

        let req = CreateSessionRequest {
            identifier: config.identifier.clone(),
            password: config.app_password.clone(),
        };

        let resp = client
            .post(format!("{pds_url}/xrpc/com.atproto.server.createSession"))
            .json(&req)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(cja::color_eyre::eyre::eyre!(
                "Bluesky login failed at {} ({}): {}",
                pds_url,
                status,
                body
            ));
        }

        let session: SessionResponse = resp.json().await?;

        Ok(Self {
            client,
            session,
            pds_url,
        })
    }

    /// Publish a note-shaped post: title, markdown body, and an external
    /// link card pointing back to the note's web page. The markdown body is
    /// converted to Bluesky-flavored plain text + rich-text facets so
    /// `[text](url)` markdown links become real clickable links and other
    /// formatting markers (bold, italic, headings, etc.) don't leak into
    /// the post as literal characters.
    ///
    /// The post text fits Bluesky's 300-grapheme limit by truncating the
    /// body if necessary; truncation drops body facets (the link metadata
    /// no longer aligns with the visible text), but the title and trailing
    /// URL facets are always preserved.
    pub async fn create_note_post(
        &self,
        title: &str,
        body_markdown: &str,
        note_url: &str,
    ) -> cja::Result<CreateRecordResponse> {
        let (text, facets) = compose_note_post(title, body_markdown, note_url);

        let record = CreateRecordRequest {
            repo: self.session.did.clone(),
            collection: "app.bsky.feed.post".to_string(),
            record: PostRecord {
                record_type: "app.bsky.feed.post".to_string(),
                text,
                created_at: chrono::Utc::now().to_rfc3339(),
                facets: if facets.is_empty() {
                    None
                } else {
                    Some(facets)
                },
                embed: Some(EmbedExternal {
                    embed_type: "app.bsky.embed.external".to_string(),
                    external: ExternalEmbed {
                        uri: note_url.to_string(),
                        title: title.to_string(),
                        description: String::new(),
                    },
                }),
            },
        };

        let resp = self
            .client
            .post(format!(
                "{}/xrpc/com.atproto.repo.createRecord",
                self.pds_url
            ))
            .bearer_auth(&self.session.access_jwt)
            .json(&record)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(cja::color_eyre::eyre::eyre!(
                "Failed to create Bluesky post ({}): {}",
                status,
                body
            ));
        }

        let response: CreateRecordResponse = resp.json().await?;
        Ok(response)
    }
}

/// Walk a markdown AST emitting Bluesky-flavored plain text and link facets.
///
/// - Markdown link `[text](url)` -> plain `text` + link facet over `text` ranges
/// - Bold / italic / strikethrough / inline-code: markers stripped, inner text kept
/// - Headings, list items, blockquotes: treated like paragraphs (markers stripped)
/// - Bare URLs in plain text get auto-linked unless already inside a markdown-link facet
/// - Output uses a single space between adjacent siblings and `\n\n` between blocks
fn markdown_to_bsky_text(body_markdown: &str) -> (String, Vec<Facet>) {
    let mut options = markdown::ParseOptions::default();
    options.constructs.gfm_strikethrough = true;
    let Ok(root) = markdown::to_mdast(body_markdown, &options) else {
        // Parse failures are rare with default options; fall back to the
        // raw body so we still post something rather than erroring out.
        return (body_markdown.trim().to_string(), Vec::new());
    };

    let mut builder = PostBodyBuilder::default();
    builder.walk(&root, /* top_level: */ true);
    let mut text = builder.text;
    // Collapse trailing whitespace from block separators.
    while text.ends_with('\n') || text.ends_with(' ') {
        text.pop();
    }

    detect_bare_urls(&text, &mut builder.facets);
    (text, builder.facets)
}

#[derive(Default)]
struct PostBodyBuilder {
    text: String,
    facets: Vec<Facet>,
}

impl PostBodyBuilder {
    fn walk(&mut self, node: &markdown::mdast::Node, top_level: bool) {
        use markdown::mdast::Node;
        match node {
            Node::Root(r) => {
                let mut first = true;
                for child in &r.children {
                    if !first {
                        self.push_block_break();
                    }
                    first = false;
                    self.walk(child, true);
                }
            }
            Node::Paragraph(p) => {
                for c in &p.children {
                    self.walk(c, false);
                }
                if !top_level {
                    self.push_block_break();
                }
            }
            Node::Heading(h) => {
                for c in &h.children {
                    self.walk(c, false);
                }
            }
            Node::Blockquote(b) => {
                for c in &b.children {
                    self.walk(c, false);
                }
            }
            Node::List(l) => {
                let mut first = true;
                for c in &l.children {
                    if !first {
                        self.text.push('\n');
                    }
                    first = false;
                    self.walk(c, false);
                }
            }
            Node::ListItem(li) => {
                self.text.push_str("- ");
                for c in &li.children {
                    self.walk(c, false);
                }
            }
            Node::Text(t) => self.text.push_str(&t.value),
            Node::Strong(s) => {
                for c in &s.children {
                    self.walk(c, false);
                }
            }
            Node::Emphasis(e) => {
                for c in &e.children {
                    self.walk(c, false);
                }
            }
            Node::Delete(d) => {
                for c in &d.children {
                    self.walk(c, false);
                }
            }
            Node::InlineCode(c) => self.text.push_str(&c.value),
            Node::Code(c) => self.text.push_str(&c.value),
            Node::Link(l) => {
                let start = self.text.len();
                for c in &l.children {
                    self.walk(c, false);
                }
                let end = self.text.len();
                if end > start {
                    self.facets.push(link_facet(start, end, &l.url));
                }
            }
            Node::Break(_) => self.text.push('\n'),
            // Images, references, MDX, HTML, math, footnotes, tables, thematic
            // breaks — drop silently. Bluesky has no analogue and they'd
            // render as garbage if echoed.
            _ => {}
        }
    }

    fn push_block_break(&mut self) {
        if !self.text.is_empty() && !self.text.ends_with("\n\n") {
            // Trim trailing whitespace before adding the break.
            while self.text.ends_with(' ') {
                self.text.pop();
            }
            self.text.push_str("\n\n");
        }
    }
}

fn link_facet(start: usize, end: usize, uri: &str) -> Facet {
    Facet {
        index: ByteSlice {
            byte_start: start,
            byte_end: end,
        },
        features: vec![FacetFeature {
            feature_type: "app.bsky.richtext.facet#link".to_string(),
            uri: uri.to_string(),
        }],
    }
}

/// Find bare URLs in `text` and append link facets for those not already
/// covered by an existing facet (so markdown-link facets aren't duplicated).
fn detect_bare_urls(text: &str, facets: &mut Vec<Facet>) {
    let trailing = |c: char| ".,!?:;)\"]'".contains(c);
    let mut cursor = 0usize;
    while let Some(found) = text[cursor..]
        .find("https://")
        .or_else(|| text[cursor..].find("http://"))
    {
        let start = cursor + found;
        let rest = &text[start..];
        let len = rest.find(|c: char| c.is_whitespace()).unwrap_or(rest.len());
        let raw = &rest[..len];
        let trimmed = raw.trim_end_matches(trailing);
        let end = start + trimmed.len();
        cursor = start + len.max(1);
        if trimmed.len() < "https://x".len() {
            continue;
        }
        if range_already_covered(facets, start, end) {
            continue;
        }
        facets.push(link_facet(start, end, trimmed));
    }
}

fn range_already_covered(facets: &[Facet], start: usize, end: usize) -> bool {
    facets
        .iter()
        .any(|f| f.index.byte_start <= start && f.index.byte_end >= end)
}

/// Compose the final post: `{title}\n\n{body}\n\n{note_url}` with facet
/// offsets shifted to point into the final string. Truncates the body to
/// fit Bluesky's 300-grapheme limit; truncation drops the body facets.
fn compose_note_post(title: &str, body_markdown: &str, note_url: &str) -> (String, Vec<Facet>) {
    let (body_text, body_facets) = markdown_to_bsky_text(body_markdown);

    // Bluesky enforces 300 graphemes (chars). Title + \n\n + body + \n\n + url.
    let overhead = title.chars().count() + note_url.chars().count() + 4;
    if overhead >= 300 {
        // Title + URL alone don't leave room for the body.
        let text = format!("{title}\n\n{note_url}");
        let url_facet = make_url_facet(&text, note_url);
        return (text, url_facet.into_iter().collect());
    }

    let max_body = 300 - overhead;
    let body_trimmed = body_text.trim();
    let body_char_count = body_trimmed.chars().count();
    let (final_body, body_facets_to_use) = if body_char_count <= max_body {
        (body_trimmed.to_string(), body_facets)
    } else {
        // Truncate; drop the body facets since their byte ranges no longer
        // match the visible characters.
        let truncated: String = body_trimmed
            .chars()
            .take(max_body.saturating_sub(1))
            .collect();
        (format!("{truncated}…"), Vec::new())
    };

    let title_bytes = title.len() + "\n\n".len();
    let mut facets: Vec<Facet> = body_facets_to_use
        .into_iter()
        .map(|f| Facet {
            index: ByteSlice {
                byte_start: f.index.byte_start + title_bytes,
                byte_end: f.index.byte_end + title_bytes,
            },
            features: f.features,
        })
        .collect();

    let text = format!("{title}\n\n{final_body}\n\n{note_url}");

    if let Some(f) = make_url_facet(&text, note_url) {
        facets.push(f);
    }

    (text, facets)
}

/// How many characters the un-truncated post would consume. Bluesky's limit
/// is 300; values above 300 mean `compose_note_post` would truncate the
/// body. CI tests use this to fail fast on notes that won't fit.
pub fn bsky_post_char_count(title: &str, body_markdown: &str, note_url: &str) -> usize {
    let (body_text, _) = markdown_to_bsky_text(body_markdown);
    // Mirrors the layout in compose_note_post: title\n\nbody\n\nurl
    title.chars().count() + 2 + body_text.trim().chars().count() + 2 + note_url.chars().count()
}

fn make_url_facet(text: &str, url: &str) -> Option<Facet> {
    text.rfind(url).map(|start| {
        let end = start + url.len();
        link_facet(start, end, url)
    })
}

pub fn at_uri_to_web_url(at_uri: &str) -> cja::Result<String> {
    let stripped = at_uri
        .strip_prefix("at://")
        .ok_or_else(|| cja::color_eyre::eyre::eyre!("Invalid AT URI: missing at:// prefix"))?;
    let (did, rest) = stripped
        .split_once('/')
        .ok_or_else(|| cja::color_eyre::eyre::eyre!("Invalid AT URI: missing collection"))?;
    let (_collection, rkey) = rest
        .split_once('/')
        .ok_or_else(|| cja::color_eyre::eyre::eyre!("Invalid AT URI: missing record key"))?;
    Ok(format!("https://bsky.app/profile/{did}/post/{rkey}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_uri_to_web_url_basic() {
        let at_uri = "at://did:plc:abc123/app.bsky.feed.post/xyz789";
        let web_url = at_uri_to_web_url(at_uri).unwrap();
        assert_eq!(
            web_url,
            "https://bsky.app/profile/did:plc:abc123/post/xyz789",
        );
    }

    #[test]
    fn at_uri_to_web_url_different_did() {
        let at_uri = "at://did:plc:ffffffffffffffff/app.bsky.feed.post/3abc123def";
        let web_url = at_uri_to_web_url(at_uri).unwrap();
        assert_eq!(
            web_url,
            "https://bsky.app/profile/did:plc:ffffffffffffffff/post/3abc123def"
        );
    }

    #[test]
    fn at_uri_to_web_url_invalid_uri() {
        let result = at_uri_to_web_url("not-an-at-uri");
        assert!(result.is_err());
    }

    #[test]
    fn at_uri_to_web_url_wrong_collection() {
        let at_uri = "at://did:plc:abc123/app.bsky.feed.like/xyz789";
        let result = at_uri_to_web_url(at_uri);
        // Still converts — we don't validate collection
        assert!(result.is_ok());
    }

    #[test]
    fn facet_byte_offsets_ascii_url() {
        let mut facets = Vec::new();
        let text = "Check out https://coreyja.com for more";
        detect_bare_urls(text, &mut facets);
        assert_eq!(facets.len(), 1);
        assert_eq!(facets[0].index.byte_start, 10);
        assert_eq!(facets[0].index.byte_end, 29);
        assert_eq!(&text[10..29], "https://coreyja.com");
    }

    #[test]
    fn facet_byte_offsets_unicode_before_url() {
        let mut facets = Vec::new();
        let text = "🦀 Check https://coreyja.com";
        detect_bare_urls(text, &mut facets);
        assert_eq!(facets.len(), 1);
        let start = facets[0].index.byte_start;
        let end = facets[0].index.byte_end;
        assert_eq!(&text[start..end], "https://coreyja.com");
        // 🦀 = 4 bytes, space = 1, "Check" = 5, space = 1 = 11 bytes before URL
        assert_eq!(start, 11);
    }

    #[test]
    fn facet_byte_offsets_url_at_end() {
        let mut facets = Vec::new();
        let text = "Read more at https://coreyja.com";
        detect_bare_urls(text, &mut facets);
        assert_eq!(facets.len(), 1);
        assert_eq!(facets[0].index.byte_end, text.len());
    }

    #[test]
    fn post_record_serializes_with_correct_type_field() {
        let record = PostRecord {
            record_type: "app.bsky.feed.post".to_string(),
            text: "Hello".to_string(),
            created_at: "2026-03-07T00:00:00Z".to_string(),
            facets: None,
            embed: None,
        };

        let json = serde_json::to_value(&record).unwrap();
        assert_eq!(json["$type"], "app.bsky.feed.post");
        assert!(
            json.get("record_type").is_none(),
            "Should not have record_type key"
        );
    }

    #[test]
    fn create_record_request_uses_camel_case() {
        let req = CreateRecordRequest {
            repo: "did:plc:test".to_string(),
            collection: "app.bsky.feed.post".to_string(),
            record: PostRecord {
                record_type: "app.bsky.feed.post".to_string(),
                text: "test".to_string(),
                created_at: "2026-03-07T00:00:00Z".to_string(),
                facets: None,
                embed: None,
            },
        };

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["record"]["createdAt"], "2026-03-07T00:00:00Z");
    }

    #[test]
    fn session_response_deserializes_camel_case() {
        let json = r#"{
            "did": "did:plc:abc123",
            "accessJwt": "eyJ..."
        }"#;

        let session: SessionResponse = serde_json::from_str(json).unwrap();
        assert_eq!(session.did, "did:plc:abc123");
        assert_eq!(session.access_jwt, "eyJ...");
    }

    // ==================== Slingshot MiniDoc parsing tests ====================

    #[test]
    fn minidoc_parses_full_response() {
        let json = r#"{
            "did": "did:plc:bg2gnrjiv6htfynausierbm2",
            "handle": "coreyja.com",
            "pds": "https://blacksky.app",
            "signing_key": "zQ3sheFdgAVcE4XxexT4F3CCiyyuKPtLCL2pFSXF4H7s6WgnV"
        }"#;
        let doc: MiniDoc = serde_json::from_str(json).unwrap();
        assert_eq!(doc.did, "did:plc:bg2gnrjiv6htfynausierbm2");
        assert_eq!(doc.handle.as_deref(), Some("coreyja.com"));
        assert_eq!(doc.pds, "https://blacksky.app");
    }

    #[test]
    fn minidoc_parses_response_without_handle() {
        // Some accounts can be DID-only (handle invalid / unset).
        let json = r#"{
            "did": "did:plc:xxx",
            "pds": "https://example.pds"
        }"#;
        let doc: MiniDoc = serde_json::from_str(json).unwrap();
        assert_eq!(doc.handle, None);
        assert_eq!(doc.pds, "https://example.pds");
    }

    #[test]
    fn minidoc_rejects_missing_required_fields() {
        // Missing pds — required for our purpose.
        let json = r#"{ "did": "did:plc:xxx" }"#;
        let result: Result<MiniDoc, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn external_embed_serializes_with_correct_type() {
        let embed = EmbedExternal {
            embed_type: "app.bsky.embed.external".to_string(),
            external: ExternalEmbed {
                uri: "https://coreyja.com/notes/test".to_string(),
                title: "Test Note".to_string(),
                description: String::new(),
            },
        };

        let json = serde_json::to_value(&embed).unwrap();
        assert_eq!(json["$type"], "app.bsky.embed.external");
        assert_eq!(json["external"]["uri"], "https://coreyja.com/notes/test");
    }

    // ==================== markdown -> bsky text + facets ====================

    fn link_uri(f: &Facet) -> &str {
        &f.features[0].uri
    }

    #[test]
    fn markdown_link_becomes_text_plus_facet() {
        let (text, facets) = markdown_to_bsky_text(
            "Built in [PR #366](https://github.com/coreyja/coreyja.com/pull/366).",
        );
        assert_eq!(text, "Built in PR #366.");
        assert_eq!(facets.len(), 1);
        let f = &facets[0];
        assert_eq!(&text[f.index.byte_start..f.index.byte_end], "PR #366");
        assert_eq!(
            link_uri(f),
            "https://github.com/coreyja/coreyja.com/pull/366"
        );
    }

    #[test]
    fn multiple_markdown_links_get_separate_facets() {
        let (text, facets) = markdown_to_bsky_text(
            "See [first](https://a.example) and [second](https://b.example) for details.",
        );
        assert_eq!(text, "See first and second for details.");
        assert_eq!(facets.len(), 2);
        assert_eq!(link_uri(&facets[0]), "https://a.example");
        assert_eq!(link_uri(&facets[1]), "https://b.example");
        // Each facet covers exactly the visible link text.
        assert_eq!(
            &text[facets[0].index.byte_start..facets[0].index.byte_end],
            "first"
        );
        assert_eq!(
            &text[facets[1].index.byte_start..facets[1].index.byte_end],
            "second"
        );
    }

    #[test]
    fn bare_url_in_plain_text_gets_a_facet() {
        let (text, facets) = markdown_to_bsky_text("Check https://coreyja.com out");
        assert_eq!(text, "Check https://coreyja.com out");
        assert_eq!(facets.len(), 1);
        assert_eq!(link_uri(&facets[0]), "https://coreyja.com");
    }

    #[test]
    fn bare_url_trailing_punctuation_excluded() {
        let (text, facets) = markdown_to_bsky_text("Visit https://coreyja.com, it's great.");
        // The link facet should not include the trailing comma.
        assert_eq!(facets.len(), 1);
        let f = &facets[0];
        assert_eq!(
            &text[f.index.byte_start..f.index.byte_end],
            "https://coreyja.com"
        );
    }

    #[test]
    fn bare_url_not_duplicated_when_inside_a_markdown_link() {
        // If the markdown link's *visible text* happens to be the URL itself,
        // we should still only emit one facet covering the visible span.
        let (text, facets) =
            markdown_to_bsky_text("[https://coreyja.com](https://coreyja.com) is the site");
        assert_eq!(text, "https://coreyja.com is the site");
        assert_eq!(facets.len(), 1);
        assert_eq!(link_uri(&facets[0]), "https://coreyja.com");
    }

    #[test]
    fn bold_italic_code_markers_are_stripped() {
        let (text, facets) =
            markdown_to_bsky_text("**bold** and *italic* and `code` survive without markers.");
        assert_eq!(text, "bold and italic and code survive without markers.");
        assert!(facets.is_empty());
    }

    #[test]
    fn paragraphs_are_separated_by_blank_lines() {
        let (text, _) = markdown_to_bsky_text("First paragraph.\n\nSecond paragraph.");
        assert_eq!(text, "First paragraph.\n\nSecond paragraph.");
    }

    #[test]
    fn heading_keeps_text_drops_markers() {
        let (text, _) = markdown_to_bsky_text("# Big Heading\n\nbody.");
        assert_eq!(text, "Big Heading\n\nbody.");
    }

    #[test]
    fn images_are_dropped() {
        let (text, facets) =
            markdown_to_bsky_text("Look ![alt](https://example.com/img.png) at this.");
        // Image syntax has no Bluesky analog; just leave the surrounding text.
        assert!(text.contains("Look"));
        assert!(text.contains("at this."));
        assert!(facets.is_empty());
    }

    #[test]
    fn unicode_offsets_in_facets_are_byte_correct() {
        let (text, facets) = markdown_to_bsky_text("🦀 see [crate](https://crates.io)");
        assert_eq!(facets.len(), 1);
        let f = &facets[0];
        // Byte slice must round-trip to "crate"
        assert_eq!(&text[f.index.byte_start..f.index.byte_end], "crate");
    }

    // ==================== compose_note_post ====================

    #[test]
    fn compose_uses_title_body_url_with_blank_lines() {
        let (text, _facets) =
            compose_note_post("Title", "Body sentence.", "https://coreyja.com/notes/x");
        assert_eq!(
            text,
            "Title\n\nBody sentence.\n\nhttps://coreyja.com/notes/x"
        );
    }

    #[test]
    fn compose_shifts_body_facets_by_title_prefix() {
        let (text, facets) = compose_note_post(
            "Title",
            "See [first](https://a.example).",
            "https://coreyja.com/notes/x",
        );
        // Find the facet pointing at https://a.example (not the trailing URL).
        let inner = facets
            .iter()
            .find(|f| link_uri(f) == "https://a.example")
            .expect("inner link facet present");
        assert_eq!(&text[inner.index.byte_start..inner.index.byte_end], "first");
    }

    #[test]
    fn compose_adds_facet_for_trailing_url() {
        let (text, facets) = compose_note_post("Title", "Body.", "https://coreyja.com/notes/x");
        let trailing = facets
            .iter()
            .find(|f| link_uri(f) == "https://coreyja.com/notes/x")
            .expect("trailing-URL facet present");
        assert_eq!(
            &text[trailing.index.byte_start..trailing.index.byte_end],
            "https://coreyja.com/notes/x"
        );
    }

    #[test]
    fn compose_truncates_body_and_drops_body_facets() {
        let long_body = "x".repeat(400);
        let body_with_link = format!("[link](https://a.example) {long_body}");
        let (text, facets) =
            compose_note_post("Title", &body_with_link, "https://coreyja.com/notes/x");
        assert!(text.chars().count() <= 300);
        // Only the trailing URL facet survives; the inner link facet is dropped
        // because the body got truncated.
        assert_eq!(facets.len(), 1);
        assert_eq!(link_uri(&facets[0]), "https://coreyja.com/notes/x");
    }

    #[test]
    fn compose_handles_title_plus_url_at_or_over_limit() {
        let title = "T".repeat(150);
        let url = format!("https://coreyja.com/{}", "y".repeat(150));
        let (text, _facets) = compose_note_post(&title, "Body that won't fit.", &url);
        // No body is included when title + url already exceed the limit.
        assert!(text.contains(&title));
        assert!(text.contains(&url));
        assert!(!text.contains("Body that won't fit."));
    }
}
