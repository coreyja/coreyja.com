//! Branded OG card SVG rendering.
//!
//! The static template lives in `server/static/og-template.svg` with `{{placeholders}}`
//! that Rust fills at request time. Quicksand `@font-face` rules with base64-embedded TTFs
//! are injected here so the source `.svg` stays human-editable.
//!
//! `og_image_url` wraps the generated SVG URL through `imgproxy` (rasterized → PNG) when
//! `IMGPROXY_URL` is configured. In local dev it points directly at the raw SVG URL.

use std::sync::LazyLock;

use base64::Engine as _;
use urlencoding::encode;

use crate::AppConfig;

const OG_TEMPLATE_SVG: &str = include_str!("../../../static/og-template.svg");
const QUICKSAND_REGULAR_TTF: &[u8] = include_bytes!("../../../static/fonts/Quicksand-Regular.ttf");
const QUICKSAND_BOLD_TTF: &[u8] = include_bytes!("../../../static/fonts/Quicksand-Bold.ttf");

const TITLE_MAX_CHARS: usize = 80;
// Tuned for Quicksand Bold 64px starting at x=80 in a 1200px-wide card. Higher values
// let medium-length titles (e.g. "Notes now syndicate to Bluesky", 30 chars) overflow
// the right edge of the card. ~28 chars fit comfortably with margin to spare.
const TITLE_SINGLE_LINE_THRESHOLD: usize = 28;

static QUICKSAND_FONT_CSS: LazyLock<String> = LazyLock::new(|| {
    let regular_b64 = base64::engine::general_purpose::STANDARD.encode(QUICKSAND_REGULAR_TTF);
    let bold_b64 = base64::engine::general_purpose::STANDARD.encode(QUICKSAND_BOLD_TTF);
    format!(
        r#"<style>
            @font-face {{ font-family: "Quicksand"; font-weight: 400;
                src: url("data:font/ttf;base64,{regular_b64}") format("truetype"); }}
            @font-face {{ font-family: "Quicksand"; font-weight: 700;
                src: url("data:font/ttf;base64,{bold_b64}") format("truetype"); }}
        </style>"#
    )
});

static YT_HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("static reqwest client builds cleanly")
});

#[derive(Debug, Clone, Copy)]
pub enum CardTag {
    Posts,
    Podcast,
    Newsletter,
    Notes,
}

impl CardTag {
    pub fn label(self) -> &'static str {
        match self {
            CardTag::Posts => "POSTS",
            CardTag::Podcast => "PODCAST",
            CardTag::Newsletter => "NEWSLETTER",
            CardTag::Notes => "NOTES",
        }
    }
}

pub struct CardData<'a> {
    pub title: &'a str,
    pub date: chrono::NaiveDate,
    pub tag: CardTag,
    /// Base64-encoded JPEG (no `data:` prefix). When `Some`, the `YouTube` `<image>` block is kept.
    pub youtube_thumbnail_b64: Option<String>,
}

/// Truncate a title to `max_chars`, breaking on the nearest word boundary and appending `…`.
fn truncate_title(title: &str, max_chars: usize) -> String {
    if title.chars().count() <= max_chars {
        return title.to_string();
    }

    let prefix: String = title.chars().take(max_chars).collect();
    let cut = prefix.rfind(char::is_whitespace).unwrap_or(prefix.len());
    let mut out = prefix[..cut].trim_end().to_string();
    out.push('…');
    out
}

/// Split a title into up to two lines, breaking on the word boundary nearest the midpoint.
///
/// Operates on `char` indices, not byte indices, so multi-byte UTF-8 codepoints (accents,
/// em-dashes, smart quotes, emoji) cannot cause a `char boundary` panic when sliced.
fn split_title_lines(title: &str) -> (String, Option<String>) {
    let chars: Vec<char> = title.chars().collect();
    if chars.len() <= TITLE_SINGLE_LINE_THRESHOLD {
        return (title.to_string(), None);
    }

    let mid = chars.len() / 2;

    // Walk left from mid (exclusive) looking for whitespace.
    let before = (0..mid).rev().find(|&i| chars[i].is_whitespace());
    // Walk right from mid (inclusive) looking for whitespace.
    let after = (mid..chars.len()).find(|&i| chars[i].is_whitespace());

    let split_at = match (before, after) {
        (Some(b), Some(a)) => {
            if mid - b <= a - mid {
                b
            } else {
                a
            }
        }
        (Some(b), None) => b,
        (None, Some(a)) => a,
        (None, None) => return (title.to_string(), None),
    };

    let line1: String = chars[..split_at]
        .iter()
        .collect::<String>()
        .trim_end()
        .to_string();
    let line2: String = chars[split_at..]
        .iter()
        .collect::<String>()
        .trim_start()
        .to_string();
    (line1, Some(line2))
}

/// Remove everything between `start_sentinel` and `end_sentinel` (inclusive).
fn strip_block(svg: &str, start_sentinel: &str, end_sentinel: &str) -> String {
    let Some(start) = svg.find(start_sentinel) else {
        return svg.to_string();
    };
    let Some(end_rel) = svg[start..].find(end_sentinel) else {
        return svg.to_string();
    };
    let end = start + end_rel + end_sentinel.len();
    let mut out = String::with_capacity(svg.len() - (end - start));
    out.push_str(&svg[..start]);
    out.push_str(&svg[end..]);
    out
}

fn substitute_title(svg: &str, title: &str) -> String {
    let truncated = truncate_title(title, TITLE_MAX_CHARS);
    let (line1, line2_opt) = split_title_lines(&truncated);

    let line1_escaped = html_escape::encode_text(&line1).into_owned();
    let svg = svg.replace("{{title_line_1}}", &line1_escaped);

    match line2_opt {
        Some(line2) => {
            let line2_escaped = html_escape::encode_text(&line2).into_owned();
            // Keep the inner <text> element; just remove the sentinel markers themselves.
            let svg = svg.replace("<!-- {{title_line_2_start}} -->", "");
            let svg = svg.replace("<!-- {{title_line_2_end}} -->", "");
            svg.replace("{{title_line_2}}", &line2_escaped)
        }
        None => strip_block(
            &svg,
            "<!-- {{title_line_2_start}} -->",
            "<!-- {{title_line_2_end}} -->",
        ),
    }
}

/// Render a branded OG card to an SVG string. Returns `image/svg+xml` content.
pub fn render_card_svg(data: &CardData<'_>) -> String {
    let svg = OG_TEMPLATE_SVG.to_string();
    let svg = svg.replace("{{font_face}}", QUICKSAND_FONT_CSS.as_str());
    let svg = svg.replace("{{logo_svg_contents}}", super::LOGO_DARK_FLAT_SVG);
    let svg = substitute_title(&svg, data.title);
    let svg = svg.replace("{{date}}", &data.date.format("%B %-d, %Y").to_string());
    let svg = svg.replace("{{tag}}", data.tag.label());

    match &data.youtube_thumbnail_b64 {
        Some(b64) => {
            let svg = svg.replace("<!-- {{youtube_block_start}} -->", "");
            let svg = svg.replace("<!-- {{youtube_block_end}} -->", "");
            svg.replace(
                "{{youtube_image_href}}",
                &format!("data:image/jpeg;base64,{b64}"),
            )
        }
        None => strip_block(
            &svg,
            "<!-- {{youtube_block_start}} -->",
            "<!-- {{youtube_block_end}} -->",
        ),
    }
}

/// Maximum chars of description shown on a publication card. Quicksand 400
/// at 28px fits roughly this many chars across the card width before
/// risking overflow on the right edge.
const PUBLICATION_DESCRIPTION_MAX_CHARS: usize = 75;

/// Render a publication-level OG card (publication name + tag + description,
/// no date).
///
/// Used as the cover blob attached to `site.standard.publication` records via
/// the standard.site sync CLI. Re-uses the same SVG template as per-post cards
/// so the publication's visual identity matches. The description is rendered
/// in the slot where per-post cards show the date.
pub fn render_publication_card_svg(title: &str, tag: CardTag, description: &str) -> String {
    let svg = OG_TEMPLATE_SVG.to_string();
    let svg = svg.replace("{{font_face}}", QUICKSAND_FONT_CSS.as_str());
    let svg = svg.replace("{{logo_svg_contents}}", super::LOGO_DARK_FLAT_SVG);
    let svg = substitute_title(&svg, title);
    let description = truncate_title(description, PUBLICATION_DESCRIPTION_MAX_CHARS);
    let description_escaped = html_escape::encode_text(&description).into_owned();
    let svg = svg.replace("{{date}}", &description_escaped);
    let svg = svg.replace("{{tag}}", tag.label());

    // Publication cards never embed a YouTube thumbnail; strip the block.
    strip_block(
        &svg,
        "<!-- {{youtube_block_start}} -->",
        "<!-- {{youtube_block_end}} -->",
    )
}

/// Absolute URL to the SVG route — uses `AppConfig.base_url`.
pub fn og_svg_url(config: &AppConfig, route_path: &str) -> String {
    config.app_url(route_path)
}

/// Absolute URL for `og:image`. Wraps via `imgproxy` (rasterized to 1200×630 PNG) when
/// `IMGPROXY_URL` is configured; otherwise returns the raw SVG URL.
pub fn og_image_url(config: &AppConfig, route_path: &str) -> String {
    let svg_url = og_svg_url(config, route_path);
    match &config.imgproxy_url {
        Some(imgproxy_base) => format!(
            "{imgproxy_base}/unsafe/rs:fill:1200:630/format:png/plain/{}",
            encode(&svg_url)
        ),
        None => svg_url,
    }
}

/// Fetch a `YouTube` thumbnail and return its base64-encoded JPEG body.
///
/// Tries `maxresdefault.jpg` first, falls back to `hqdefault.jpg` on non-200. Returns
/// `None` on any failure (logged) or when `youtube_id` is empty.
pub async fn fetch_youtube_thumbnail_b64(youtube_id: &str) -> Option<String> {
    if youtube_id.is_empty() {
        return None;
    }

    let urls = [
        format!("https://i.ytimg.com/vi/{youtube_id}/maxresdefault.jpg"),
        format!("https://i.ytimg.com/vi/{youtube_id}/hqdefault.jpg"),
    ];

    for url in &urls {
        match YT_HTTP_CLIENT.get(url).send().await {
            Ok(resp) if resp.status().is_success() => match resp.bytes().await {
                Ok(bytes) => {
                    return Some(base64::engine::general_purpose::STANDARD.encode(&bytes));
                }
                Err(e) => {
                    tracing::warn!(
                        youtube_id = %youtube_id,
                        error = ?e,
                        "Failed to read YouTube thumbnail body",
                    );
                }
            },
            Ok(resp) => {
                tracing::warn!(
                    youtube_id = %youtube_id,
                    status = %resp.status(),
                    url = %url,
                    "YouTube thumbnail fetch returned non-success",
                );
            }
            Err(e) => {
                tracing::warn!(
                    youtube_id = %youtube_id,
                    error = ?e,
                    url = %url,
                    "YouTube thumbnail HTTP request failed",
                );
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::normalize_imgproxy_url;
    use url::Url;

    fn raw_config(imgproxy_url: Option<String>) -> AppConfig {
        AppConfig {
            base_url: Url::parse("https://coreyja.com").unwrap(),
            imgproxy_url,
        }
    }

    fn sample_data() -> CardData<'static> {
        CardData {
            title: "Sample Title",
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
            tag: CardTag::Posts,
            youtube_thumbnail_b64: None,
        }
    }

    #[test]
    fn truncate_title_short_returns_unchanged() {
        assert_eq!(truncate_title("Short title", 80), "Short title");
    }

    #[test]
    fn truncate_title_long_truncates_at_word_boundary_with_ellipsis() {
        let long = "a".repeat(40) + " " + &"b".repeat(60);
        let out = truncate_title(&long, 80);
        assert!(out.ends_with('…'));
        // Should break at the space rather than cutting mid-word.
        assert!(out.starts_with(&"a".repeat(40)));
    }

    #[test]
    fn truncate_title_at_exact_boundary_does_not_append_ellipsis() {
        let exact = "x".repeat(80);
        assert_eq!(truncate_title(&exact, 80), exact);
    }

    #[test]
    fn split_title_lines_under_threshold_returns_one_line() {
        let (a, b) = split_title_lines("Short title under threshold");
        assert_eq!(a, "Short title under threshold");
        assert!(b.is_none());
    }

    #[test]
    fn split_title_lines_at_exact_threshold_returns_one_line() {
        let title = "x".repeat(TITLE_SINGLE_LINE_THRESHOLD);
        let (a, b) = split_title_lines(&title);
        assert_eq!(a, title);
        assert!(b.is_none());
    }

    #[test]
    fn split_title_lines_wraps_real_world_30_char_title() {
        // Regression: "Notes now syndicate to Bluesky" (30 chars) rendered as one line
        // overflowed the right edge of the 1200px card with the previous 40-char threshold.
        let title = "Notes now syndicate to Bluesky";
        assert!(title.chars().count() > TITLE_SINGLE_LINE_THRESHOLD);
        let (_, line2) = split_title_lines(title);
        assert!(
            line2.is_some(),
            "30-char title should wrap to two lines to stay within card bounds"
        );
    }

    #[test]
    fn split_title_lines_over_threshold_splits_at_nearest_midpoint_word_boundary() {
        let title = "This is a fairly long blog post title for testing";
        let (a, b) = split_title_lines(title);
        assert!(b.is_some());
        let b = b.unwrap();
        // Reassembled they should equal the original (modulo whitespace at the split).
        assert_eq!(format!("{a} {b}"), title);
        // Lines should be reasonably balanced — neither empty, both meaningful.
        assert!(!a.is_empty());
        assert!(!b.is_empty());
    }

    #[test]
    fn render_card_svg_substitutes_all_placeholders() {
        let svg = render_card_svg(&sample_data());
        assert!(
            !svg.contains("{{"),
            "All placeholders should be substituted; got:\n{svg}"
        );
    }

    #[test]
    fn render_card_svg_strips_line_2_block_when_title_fits_one_line() {
        let svg = render_card_svg(&sample_data());
        assert!(
            !svg.contains("{{title_line_2}}"),
            "title_line_2 placeholder should be removed"
        );
        // The line-2 <text> element should be entirely gone.
        let line2_count = svg.matches("font-size=\"64\"").count();
        assert_eq!(
            line2_count, 1,
            "Only one line of title text should remain when title fits one line"
        );
    }

    #[test]
    fn render_card_svg_emits_line_2_when_title_spans_two_lines() {
        let data = CardData {
            title: "This is a long enough blog post title to force two-line wrapping",
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
            tag: CardTag::Posts,
            youtube_thumbnail_b64: None,
        };
        let svg = render_card_svg(&data);
        assert!(!svg.contains("{{title_line_2}}"));
        assert!(!svg.contains("{{title_line_2_start}}"));
        assert!(!svg.contains("{{title_line_2_end}}"));
        let line2_count = svg.matches("font-size=\"64\"").count();
        assert_eq!(
            line2_count, 2,
            "Both lines of title text should be present when title wraps"
        );
    }

    #[test]
    fn render_card_svg_strips_youtube_block_when_none() {
        let svg = render_card_svg(&sample_data());
        assert!(!svg.contains("{{youtube_image_href}}"));
        assert!(!svg.contains("<image"));
    }

    #[test]
    fn render_card_svg_includes_data_uri_when_thumbnail_present() {
        let data = CardData {
            title: "Sample",
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
            tag: CardTag::Podcast,
            youtube_thumbnail_b64: Some("ZmFrZWltYWdl".to_string()),
        };
        let svg = render_card_svg(&data);
        assert!(svg.contains("data:image/jpeg;base64,ZmFrZWltYWdl"));
        assert!(svg.contains("<image"));
    }

    #[test]
    fn render_card_svg_escapes_xml_special_chars_in_title() {
        let data = CardData {
            title: "Title with <html> & \"quotes\"",
            date: chrono::NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
            tag: CardTag::Posts,
            youtube_thumbnail_b64: None,
        };
        let svg = render_card_svg(&data);
        assert!(svg.contains("&lt;html&gt;"));
        assert!(svg.contains("&amp;"));
    }

    #[test]
    fn og_image_url_wraps_with_imgproxy_when_configured() {
        let cfg = raw_config(Some("https://img.coreyja.com".to_string()));
        let out = og_image_url(&cfg, "/og/posts/abc.svg");
        assert!(
            out.starts_with("https://img.coreyja.com/unsafe/rs:fill:1200:630/format:png/plain/")
        );
        assert!(out.contains("https%3A%2F%2Fcoreyja.com%2Fog%2Fposts%2Fabc.svg"));
    }

    #[test]
    fn og_image_url_returns_raw_svg_url_when_imgproxy_none() {
        let cfg = raw_config(None);
        let out = og_image_url(&cfg, "/og/posts/abc.svg");
        assert_eq!(out, "https://coreyja.com/og/posts/abc.svg");
    }

    #[test]
    fn og_image_url_handles_imgproxy_with_or_without_trailing_slash() {
        // Normalize externally — this mirrors the work that AppConfig::from_env does
        // and avoids mutating process-global env vars from the test.
        let a = raw_config(Some(normalize_imgproxy_url("https://img.coreyja.com")));
        let b = raw_config(Some(normalize_imgproxy_url("https://img.coreyja.com/")));
        let url_a = og_image_url(&a, "/og/posts/abc.svg");
        let url_b = og_image_url(&b, "/og/posts/abc.svg");
        assert_eq!(url_a, url_b);
        // Exactly one slash between the host and `/unsafe/`.
        assert!(url_a.contains("https://img.coreyja.com/unsafe/"));
        assert!(!url_a.contains("https://img.coreyja.com//unsafe/"));
    }

    #[test]
    fn split_title_lines_handles_non_ascii_at_byte_midpoint() {
        // 42 chars of "é" with a space at index 21. Each "é" is 2 bytes; the byte-len midpoint
        // would land mid-codepoint and panic with the old byte-index implementation.
        let title = format!("{} {}", "é".repeat(21), "é".repeat(20));
        let (line1, line2) = split_title_lines(&title);
        assert_eq!(line1.chars().count(), 21);
        let line2 = line2.expect("title over threshold should produce two lines");
        assert_eq!(line2.chars().count(), 20);
    }

    #[test]
    fn split_title_lines_handles_emoji_at_byte_midpoint() {
        // 4-byte codepoints with whitespace far from the byte midpoint — would also panic
        // under byte indexing.
        let title =
            "🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀 🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀";
        let (line1, line2) = split_title_lines(title);
        assert!(line2.is_some(), "should split into two lines");
        // Each line should contain only the rocket emoji, no whitespace at the seam.
        assert!(!line1.is_empty());
        assert!(!line2.unwrap().is_empty());
    }
}
