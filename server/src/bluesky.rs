// Test scaffold for Bluesky API client module - BLOG-6a79edf75fad4d29
// The implementation agent should add BlueskyConfig, BlueskyClient, AT Protocol types,
// and the at_uri_to_web_url helper above this test block, then un-ignore tests.

#[cfg(test)]
mod tests {
    // Tests will use: super::*
    // Required types: BlueskyConfig, BlueskyClient, CreateRecordRequest, PostRecord, Facet, etc.

    // ==================== at_uri_to_web_url tests ====================

    #[test]
    #[ignore = "Requires at_uri_to_web_url function"]
    fn at_uri_to_web_url_basic() {
        let at_uri = "at://did:plc:abc123/app.bsky.feed.post/xyz789";
        let web_url = super::at_uri_to_web_url(at_uri).unwrap();
        assert_eq!(
            web_url, "https://bsky.app/profile/did:plc:abc123/post/xyz789",
            "Should convert AT URI to bsky.app web URL"
        );
    }

    #[test]
    #[ignore = "Requires at_uri_to_web_url function"]
    fn at_uri_to_web_url_different_did() {
        let at_uri = "at://did:plc:ffffffffffffffff/app.bsky.feed.post/3abc123def";
        let web_url = super::at_uri_to_web_url(at_uri).unwrap();
        assert_eq!(
            web_url,
            "https://bsky.app/profile/did:plc:ffffffffffffffff/post/3abc123def"
        );
    }

    #[test]
    #[ignore = "Requires at_uri_to_web_url function"]
    fn at_uri_to_web_url_invalid_uri() {
        // Invalid AT URIs should return an error
        let result = super::at_uri_to_web_url("not-an-at-uri");
        assert!(result.is_err(), "Invalid AT URI should return an error");
    }

    #[test]
    #[ignore = "Requires at_uri_to_web_url function"]
    fn at_uri_to_web_url_wrong_collection() {
        // Only app.bsky.feed.post should be supported
        let at_uri = "at://did:plc:abc123/app.bsky.feed.like/xyz789";
        let result = super::at_uri_to_web_url(at_uri);
        // This could either error or still convert — depends on implementation.
        // At minimum it should not panic.
        let _ = result;
    }

    // ==================== Facet byte offset tests ====================

    #[test]
    #[ignore = "Requires facet construction helpers"]
    fn facet_byte_offsets_ascii_url() {
        // For ASCII text, byte offsets and char offsets are the same
        let text = "Check out https://coreyja.com for more";
        let url = "https://coreyja.com";
        let start = text.find(url).unwrap();
        let end = start + url.len();

        // Byte offsets should match string positions for ASCII
        assert_eq!(start, 10, "URL should start at byte 10");
        assert_eq!(end, 29, "URL should end at byte 29");
    }

    #[test]
    #[ignore = "Requires facet construction helpers"]
    fn facet_byte_offsets_unicode_before_url() {
        // When there's multi-byte unicode before the URL, byte offsets differ from char offsets
        let text = "🦀 Check https://coreyja.com";
        let url = "https://coreyja.com";

        let byte_start = text.find(url).unwrap();
        let byte_end = byte_start + url.len();

        // "🦀 Check " is 4 + 1 + 5 + 1 = 11 bytes (🦀 is 4 bytes in UTF-8)
        // But only 8 characters
        let char_start = text.chars().take_while(|_| {
            // This is just to show the difference
            true
        });
        let _ = char_start;

        // The byte offset should account for the 4-byte emoji
        assert_eq!(
            byte_start, 12,
            "URL byte offset should account for multi-byte emoji"
        );
        assert_eq!(byte_end, 31);
    }

    #[test]
    #[ignore = "Requires facet construction helpers"]
    fn facet_byte_offsets_url_at_end() {
        let text = "Read more at https://coreyja.com";
        let url = "https://coreyja.com";
        let start = text.find(url).unwrap();
        let end = start + url.len();

        assert_eq!(
            end,
            text.len(),
            "URL at end should have end offset equal to text length"
        );
    }

    // ==================== Request serialization tests ====================

    #[test]
    #[ignore = "Requires PostRecord and CreateRecordRequest types"]
    fn post_record_serializes_with_correct_type_field() {
        // The $type field must serialize as "$type" not "type"
        // PostRecord should have #[serde(rename = "$type")] pub record_type: String
        // with value "app.bsky.feed.post"

        // This test verifies the serde rename attribute works correctly
        // Implementation should create a PostRecord and serialize it to JSON,
        // then check that the JSON contains "$type": "app.bsky.feed.post"
    }

    #[test]
    #[ignore = "Requires CreateRecordRequest type"]
    fn create_record_request_uses_camel_case() {
        // AT Protocol uses camelCase for JSON fields
        // CreateRecordRequest should have #[serde(rename_all = "camelCase")]
        // Fields like access_jwt should serialize as "accessJwt"
    }

    #[test]
    #[ignore = "Requires SessionResponse type"]
    fn session_response_deserializes_camel_case() {
        // The session response from bsky.social uses camelCase
        let json = r#"{
            "did": "did:plc:abc123",
            "accessJwt": "eyJ...",
            "handle": "coreyja.com"
        }"#;

        // Should deserialize into SessionResponse with snake_case fields
        // let session: super::SessionResponse = serde_json::from_str(json).unwrap();
        // assert_eq!(session.did, "did:plc:abc123");
        // assert_eq!(session.access_jwt, "eyJ...");
        // assert_eq!(session.handle, "coreyja.com");
    }

    #[test]
    #[ignore = "Requires embed types for website cards"]
    fn external_embed_serializes_with_correct_type() {
        // Embed type should be "app.bsky.embed.external"
        // The embed should contain a uri and title for the website card
    }
}
