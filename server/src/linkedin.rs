#![allow(clippy::doc_markdown)]

use cja::color_eyre::eyre::Context;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::encrypt;

/// LinkedIn REST API version pin. Versions deprecate ~12 months after release.
/// Update at PR-open time to a version released within the last 6 months.
/// See <https://learn.microsoft.com/en-us/linkedin/marketing/versioning>
pub const LINKEDIN_VERSION_HEADER: &str = "202602";

#[derive(Debug, Clone)]
pub struct LinkedInConfig {
    pub client_id: String,
    pub client_secret: String,
}

impl LinkedInConfig {
    pub fn from_env() -> cja::Result<Self> {
        let client_id =
            std::env::var("LINKEDIN_CLIENT_ID").context("LINKEDIN_CLIENT_ID env var missing")?;
        let client_secret = std::env::var("LINKEDIN_CLIENT_SECRET")
            .context("LINKEDIN_CLIENT_SECRET env var missing")?;
        Ok(Self {
            client_id,
            client_secret,
        })
    }

    pub fn from_env_optional() -> cja::Result<Option<Self>> {
        let id = std::env::var("LINKEDIN_CLIENT_ID").ok();
        let secret = std::env::var("LINKEDIN_CLIENT_SECRET").ok();
        match (id, secret) {
            (None, None) => Ok(None),
            (Some(client_id), Some(client_secret)) => Ok(Some(Self {
                client_id,
                client_secret,
            })),
            (Some(_), None) => Err(cja::color_eyre::eyre::eyre!(
                "Partial LinkedIn config: LINKEDIN_CLIENT_ID set but LINKEDIN_CLIENT_SECRET missing"
            )),
            (None, Some(_)) => Err(cja::color_eyre::eyre::eyre!(
                "Partial LinkedIn config: LINKEDIN_CLIENT_SECRET set but LINKEDIN_CLIENT_ID missing"
            )),
        }
    }
}

pub(crate) struct LinkedInUserRow {
    pub linkedin_user_id: Uuid,
    pub encrypted_access_token: Vec<u8>,
    pub encrypted_refresh_token: Vec<u8>,
    pub access_token_expires_at: chrono::DateTime<chrono::Utc>,
    pub refresh_token_expires_at: chrono::DateTime<chrono::Utc>,
    pub external_linkedin_id: String,
}

#[derive(Debug, Deserialize)]
struct LinkedInRefreshTokenResponse {
    access_token: String,
    expires_in: i64,
    refresh_token: String,
    refresh_token_expires_in: i64,
    #[allow(dead_code)]
    scope: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePostResponse {
    /// The post URN (e.g., `urn:li:share:1234`, `urn:li:ugcPost:abc`).
    pub urn: String,
}

#[derive(Debug, Serialize)]
struct CreatePostRequest<'a> {
    author: &'a str,
    commentary: &'a str,
    visibility: &'static str,
    distribution: Distribution,
    #[serde(rename = "lifecycleState")]
    lifecycle_state: &'static str,
    #[serde(rename = "isReshareDisabledByAuthor")]
    is_reshare_disabled_by_author: bool,
}

#[derive(Debug, Serialize)]
struct Distribution {
    #[serde(rename = "feedDistribution")]
    feed_distribution: &'static str,
    #[serde(rename = "targetEntities")]
    target_entities: Vec<String>,
    #[serde(rename = "thirdPartyDistributionChannels")]
    third_party_distribution_channels: Vec<String>,
}

pub struct LinkedInClient {
    http: reqwest::Client,
    access_token: String,
    author_urn: String,
}

impl LinkedInClient {
    /// Build a client from environment + DB. Reads the single `LinkedInUsers`
    /// row, refreshes the access token if it's expired (or within 5 min of
    /// expiring), and returns a ready-to-use client.
    pub async fn from_db_env() -> cja::Result<Self> {
        let pool = db::setup_db_pool().await?;
        let encrypt_config = encrypt::Config::from_env()?;
        let linkedin_config = LinkedInConfig::from_env()?;

        let row = sqlx::query_as!(
            LinkedInUserRow,
            r#"
            SELECT
                linkedin_user_id,
                encrypted_access_token,
                encrypted_refresh_token,
                access_token_expires_at,
                refresh_token_expires_at,
                external_linkedin_id
            FROM LinkedInUsers
            LIMIT 1
            "#
        )
        .fetch_optional(&pool)
        .await?;

        let row = row.ok_or_else(|| {
            cja::color_eyre::eyre::eyre!(
                "No LinkedIn user found — visit /admin/auth/linkedin to authorize"
            )
        })?;

        let access_token =
            if row.access_token_expires_at < chrono::Utc::now() + chrono::Duration::minutes(5) {
                refresh_linkedin_token(&pool, &encrypt_config, &linkedin_config, &row).await?
            } else {
                encrypt_config.decrypt(&row.encrypted_access_token)?
            };

        let author_urn = format!("urn:li:person:{}", row.external_linkedin_id);

        Ok(Self {
            http: reqwest::Client::new(),
            access_token,
            author_urn,
        })
    }

    /// POST a plain-text post to LinkedIn's `/rest/posts` endpoint. Returns
    /// the post URN, read from the `x-restli-id` header (with the JSON `id`
    /// field as a fallback).
    pub async fn create_text_post(&self, commentary: &str) -> cja::Result<CreatePostResponse> {
        let body = CreatePostRequest {
            author: &self.author_urn,
            commentary,
            visibility: "PUBLIC",
            distribution: Distribution {
                feed_distribution: "MAIN_FEED",
                target_entities: Vec::new(),
                third_party_distribution_channels: Vec::new(),
            },
            lifecycle_state: "PUBLISHED",
            is_reshare_disabled_by_author: false,
        };

        let resp = self
            .http
            .post("https://api.linkedin.com/rest/posts")
            .bearer_auth(&self.access_token)
            .header("LinkedIn-Version", LINKEDIN_VERSION_HEADER)
            .header("X-Restli-Protocol-Version", "2.0.0")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(cja::color_eyre::eyre::eyre!(
                "LinkedIn /rest/posts failed ({status}): {body}"
            ));
        }

        let header_urn = resp
            .headers()
            .get("x-restli-id")
            .and_then(|v| v.to_str().ok())
            .map(std::string::ToString::to_string);

        // Parse the JSON body even if we have the header — gives us a fallback
        // and surfaces a malformed payload before we silently return an empty URN.
        let body_text = resp.text().await.unwrap_or_default();
        let json_urn: Option<String> = serde_json::from_str::<serde_json::Value>(&body_text)
            .ok()
            .and_then(|v| {
                v.get("id")
                    .and_then(|id| id.as_str().map(std::string::ToString::to_string))
            });

        let urn = header_urn.or(json_urn).ok_or_else(|| {
            cja::color_eyre::eyre::eyre!(
                "LinkedIn /rest/posts response missing both x-restli-id header and id field"
            )
        })?;

        Ok(CreatePostResponse { urn })
    }
}

/// Refresh the stored LinkedIn access token. Updates the DB row with both
/// the new access token and the new refresh token (LinkedIn rotates both).
/// Returns the new plaintext access token.
#[tracing::instrument(name = "refresh_linkedin_token", skip_all, fields(linkedin_user_id = %row.linkedin_user_id))]
pub(crate) async fn refresh_linkedin_token(
    pool: &PgPool,
    encrypt_config: &encrypt::Config,
    config: &LinkedInConfig,
    row: &LinkedInUserRow,
) -> cja::Result<String> {
    let refresh_token = encrypt_config.decrypt(&row.encrypted_refresh_token)?;

    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", &refresh_token),
        ("client_id", &config.client_id),
        ("client_secret", &config.client_secret),
    ];

    let resp = client
        .post("https://www.linkedin.com/oauth/v2/accessToken")
        .form(&params)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(cja::color_eyre::eyre::eyre!(
            "LinkedIn refresh token request failed ({status}): {body}"
        ));
    }

    let token_data: LinkedInRefreshTokenResponse = resp.json().await?;

    let encrypted_access = encrypt_config.encrypt(&token_data.access_token)?;
    let encrypted_refresh = encrypt_config.encrypt(&token_data.refresh_token)?;

    let access_expires_at = chrono::Utc::now() + chrono::Duration::seconds(token_data.expires_in);
    let refresh_expires_at =
        chrono::Utc::now() + chrono::Duration::seconds(token_data.refresh_token_expires_in);

    sqlx::query!(
        r#"
        UPDATE LinkedInUsers
        SET
            encrypted_access_token = $1,
            access_token_expires_at = $2,
            encrypted_refresh_token = $3,
            refresh_token_expires_at = $4,
            updated_at = NOW()
        WHERE linkedin_user_id = $5
        "#,
        encrypted_access,
        access_expires_at,
        encrypted_refresh,
        refresh_expires_at,
        row.linkedin_user_id,
    )
    .execute(pool)
    .await?;

    Ok(token_data.access_token)
}

/// Extract the first paragraph of the markdown body as plain text.
/// `body_markdown` MUST have the YAML frontmatter already stripped upstream
/// — ParseOptions::default() is correct for body-only input.
pub fn extract_first_paragraph(body_markdown: &str) -> String {
    use markdown::mdast::Node;
    let options = markdown::ParseOptions::default();
    let Ok(root) = markdown::to_mdast(body_markdown, &options) else {
        return String::new();
    };
    let Node::Root(root) = root else {
        return String::new();
    };

    for child in &root.children {
        if let Node::Paragraph(p) = child {
            let mut out = String::new();
            render_inline(&Node::Paragraph(p.clone()), &mut out);
            return out;
        }
    }
    String::new()
}

fn render_inline(node: &markdown::mdast::Node, out: &mut String) {
    use markdown::mdast::Node;
    match node {
        Node::Paragraph(p) => {
            for c in &p.children {
                render_inline(c, out);
            }
        }
        Node::Text(t) => out.push_str(&t.value),
        Node::Strong(s) => {
            for c in &s.children {
                render_inline(c, out);
            }
        }
        Node::Emphasis(e) => {
            for c in &e.children {
                render_inline(c, out);
            }
        }
        Node::Delete(d) => {
            for c in &d.children {
                render_inline(c, out);
            }
        }
        Node::Link(l) => {
            for c in &l.children {
                render_inline(c, out);
            }
        }
        Node::InlineCode(c) => out.push_str(&c.value),
        Node::Break(_) => out.push('\n'),
        // Images, HTML, footnote/link references, MDX, math: skip.
        _ => {}
    }
}

/// Compose the LinkedIn post body: custom-or-first-paragraph + footer.
/// If `custom_or_first_paragraph` is empty after trim, substitutes `title`
/// so LinkedIn doesn't reject the post with a 422 for empty commentary.
/// The footer is appended unconditionally — this may duplicate a footer if
/// the author wrote one themselves; that's an acceptable trade.
pub fn compose_linkedin_body(
    custom_or_first_paragraph: &str,
    title: &str,
    canonical_url: &str,
) -> String {
    let body = if custom_or_first_paragraph.trim().is_empty() {
        title
    } else {
        custom_or_first_paragraph
    };
    format!("{body}\n\nNew post on coreyja.com\n{canonical_url}")
}

/// Convert any `urn:li:<type>:<id>` URN into a LinkedIn web URL pointing at
/// the post. Works for `share`, `ugcPost`, and `activity` URNs alike.
pub fn linkedin_urn_to_web_url(urn: &str) -> cja::Result<String> {
    if !urn.starts_with("urn:li:") {
        return Err(cja::color_eyre::eyre::eyre!(
            "Invalid LinkedIn URN (missing urn:li: prefix): {urn}"
        ));
    }
    // Must have at least 3 colon-separated segments after "urn:li:".
    let segments: Vec<&str> = urn.split(':').collect();
    if segments.len() < 4 {
        return Err(cja::color_eyre::eyre::eyre!(
            "Invalid LinkedIn URN (too few segments): {urn}"
        ));
    }
    Ok(format!("https://www.linkedin.com/feed/update/{urn}/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== extract_first_paragraph ====================

    #[test]
    fn extract_first_paragraph_basic() {
        let md = "# Heading\n\nFirst para.\n\nSecond para.";
        assert_eq!(extract_first_paragraph(md), "First para.");
    }

    #[test]
    fn extract_first_paragraph_strips_inline_markers() {
        let md = "**Hello** _world_ [link](https://x)";
        assert_eq!(extract_first_paragraph(md), "Hello world link");
    }

    #[test]
    fn extract_first_paragraph_keeps_inline_code() {
        let md = "text with `code` survives";
        assert_eq!(extract_first_paragraph(md), "text with code survives");
    }

    #[test]
    fn extract_first_paragraph_empty_body() {
        assert_eq!(extract_first_paragraph(""), "");
    }

    #[test]
    fn extract_first_paragraph_only_heading() {
        assert_eq!(extract_first_paragraph("# Just a heading"), "");
    }

    #[test]
    fn extract_first_paragraph_image_dropped() {
        let md = "Look ![alt](https://x/img.png) here";
        let out = extract_first_paragraph(md);
        assert!(!out.contains("!["));
        assert!(!out.contains("https://x/img.png"));
        assert!(out.contains("Look"));
        assert!(out.contains("here"));
    }

    // ==================== compose_linkedin_body ====================

    #[test]
    fn compose_linkedin_body_appends_footer() {
        let out = compose_linkedin_body("hi", "Title", "https://coreyja.com/x");
        assert_eq!(out, "hi\n\nNew post on coreyja.com\nhttps://coreyja.com/x");
    }

    #[test]
    fn compose_linkedin_body_falls_back_to_title_when_empty() {
        let out = compose_linkedin_body("   ", "My Title", "https://coreyja.com/x");
        assert_eq!(
            out,
            "My Title\n\nNew post on coreyja.com\nhttps://coreyja.com/x"
        );
    }

    // ==================== linkedin_urn_to_web_url ====================

    #[test]
    fn linkedin_urn_to_web_url_share() {
        assert_eq!(
            linkedin_urn_to_web_url("urn:li:share:1234").unwrap(),
            "https://www.linkedin.com/feed/update/urn:li:share:1234/"
        );
    }

    #[test]
    fn linkedin_urn_to_web_url_ugc_post() {
        assert_eq!(
            linkedin_urn_to_web_url("urn:li:ugcPost:abc").unwrap(),
            "https://www.linkedin.com/feed/update/urn:li:ugcPost:abc/"
        );
    }

    #[test]
    fn linkedin_urn_to_web_url_activity() {
        assert_eq!(
            linkedin_urn_to_web_url("urn:li:activity:7").unwrap(),
            "https://www.linkedin.com/feed/update/urn:li:activity:7/"
        );
    }

    #[test]
    fn linkedin_urn_to_web_url_invalid_returns_err() {
        assert!(linkedin_urn_to_web_url("not-a-urn").is_err());
        assert!(linkedin_urn_to_web_url("urn:other:x:y").is_err());
    }

    // ==================== from_env_optional ====================
    //
    // Consolidated into a single test that toggles env vars sequentially.
    // `cargo test` runs tests in parallel by default, so two separate
    // env-mutating tests could race and observe each other's writes. Keeping
    // all three assertions in one function gives a stable execution order.
    // Adding `serial_test` as a dev-dep would be the alternative; one test
    // function is cheaper.

    #[test]
    fn from_env_optional_handles_all_combinations() {
        let prev_id = std::env::var("LINKEDIN_CLIENT_ID").ok();
        let prev_secret = std::env::var("LINKEDIN_CLIENT_SECRET").ok();

        // 1. Both unset → Ok(None)
        std::env::remove_var("LINKEDIN_CLIENT_ID");
        std::env::remove_var("LINKEDIN_CLIENT_SECRET");
        let result = LinkedInConfig::from_env_optional();
        let none_result = result.expect("both unset should be Ok");
        assert!(none_result.is_none(), "both unset should be Ok(None)");

        // 2. Both set → Ok(Some(_))
        std::env::set_var("LINKEDIN_CLIENT_ID", "id-test");
        std::env::set_var("LINKEDIN_CLIENT_SECRET", "secret-test");
        let some_result = LinkedInConfig::from_env_optional()
            .expect("both set should be Ok")
            .expect("both set should be Some");
        assert_eq!(some_result.client_id, "id-test");
        assert_eq!(some_result.client_secret, "secret-test");

        // 3. Only ID set → Err (partial-config trap)
        std::env::set_var("LINKEDIN_CLIENT_ID", "only-id");
        std::env::remove_var("LINKEDIN_CLIENT_SECRET");
        assert!(
            LinkedInConfig::from_env_optional().is_err(),
            "only ID set should be Err"
        );

        // 4. Only secret set → Err (partial-config trap, other direction)
        std::env::remove_var("LINKEDIN_CLIENT_ID");
        std::env::set_var("LINKEDIN_CLIENT_SECRET", "only-secret");
        assert!(
            LinkedInConfig::from_env_optional().is_err(),
            "only secret set should be Err"
        );

        // Restore prior values.
        match prev_id {
            Some(v) => std::env::set_var("LINKEDIN_CLIENT_ID", v),
            None => std::env::remove_var("LINKEDIN_CLIENT_ID"),
        }
        match prev_secret {
            Some(v) => std::env::set_var("LINKEDIN_CLIENT_SECRET", v),
            None => std::env::remove_var("LINKEDIN_CLIENT_SECRET"),
        }
    }
}
