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
        let req = CreateSessionRequest {
            identifier: config.identifier.clone(),
            password: config.app_password.clone(),
        };

        let resp = client
            .post("https://bsky.social/xrpc/com.atproto.server.createSession")
            .json(&req)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(cja::color_eyre::eyre::eyre!(
                "Bluesky login failed ({}): {}",
                status,
                body
            ));
        }

        let session: SessionResponse = resp.json().await?;

        Ok(Self { client, session })
    }

    pub async fn create_post(
        &self,
        text: &str,
        url: &str,
        title: &str,
    ) -> cja::Result<CreateRecordResponse> {
        let facets = build_link_facets(text);

        let record = CreateRecordRequest {
            repo: self.session.did.clone(),
            collection: "app.bsky.feed.post".to_string(),
            record: PostRecord {
                record_type: "app.bsky.feed.post".to_string(),
                text: text.to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                facets: if facets.is_empty() {
                    None
                } else {
                    Some(facets)
                },
                embed: Some(EmbedExternal {
                    embed_type: "app.bsky.embed.external".to_string(),
                    external: ExternalEmbed {
                        uri: url.to_string(),
                        title: title.to_string(),
                        description: String::new(),
                    },
                }),
            },
        };

        let resp = self
            .client
            .post("https://bsky.social/xrpc/com.atproto.repo.createRecord")
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

fn build_link_facets(text: &str) -> Vec<Facet> {
    let mut facets = Vec::new();
    let mut search_start = 0;

    for word in text.split_whitespace() {
        if word.starts_with("https://") || word.starts_with("http://") {
            if let Some(pos) = text[search_start..].find(word) {
                let start = search_start + pos;
                let end = start + word.len();
                facets.push(Facet {
                    index: ByteSlice {
                        byte_start: start,
                        byte_end: end,
                    },
                    features: vec![FacetFeature {
                        feature_type: "app.bsky.richtext.facet#link".to_string(),
                        uri: word.to_string(),
                    }],
                });
                search_start = end;
            }
        }
    }

    facets
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
        let text = "Check out https://coreyja.com for more";
        let facets = build_link_facets(text);
        assert_eq!(facets.len(), 1);
        assert_eq!(facets[0].index.byte_start, 10);
        assert_eq!(facets[0].index.byte_end, 29);
        assert_eq!(&text[10..29], "https://coreyja.com");
    }

    #[test]
    fn facet_byte_offsets_unicode_before_url() {
        let text = "🦀 Check https://coreyja.com";
        let facets = build_link_facets(text);
        assert_eq!(facets.len(), 1);
        let start = facets[0].index.byte_start;
        let end = facets[0].index.byte_end;
        assert_eq!(&text[start..end], "https://coreyja.com");
        // 🦀 = 4 bytes, space = 1, "Check" = 5, space = 1 = 11 bytes before URL
        assert_eq!(start, 11);
    }

    #[test]
    fn facet_byte_offsets_url_at_end() {
        let text = "Read more at https://coreyja.com";
        let facets = build_link_facets(text);
        assert_eq!(facets.len(), 1);
        assert_eq!(facets[0].index.byte_end, text.len());
    }

    #[test]
    fn facet_byte_offsets_duplicate_urls() {
        let text = "Visit https://coreyja.com and then https://coreyja.com again";
        let facets = build_link_facets(text);
        assert_eq!(facets.len(), 2);
        let start1 = facets[0].index.byte_start;
        let end1 = facets[0].index.byte_end;
        let start2 = facets[1].index.byte_start;
        let end2 = facets[1].index.byte_end;
        assert_eq!(&text[start1..end1], "https://coreyja.com");
        assert_eq!(&text[start2..end2], "https://coreyja.com");
        assert_ne!(
            start1, start2,
            "Duplicate URLs should have different offsets"
        );
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
}
