use std::{collections::BTreeMap, sync::Arc};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::NaiveTime;
use maud::{html, Markup, Render};
use posts::podcast::{PodcastEpisode, PodcastEpisodes};
use rss::extension::{
    itunes::{ITunesChannelExtensionBuilder, ITunesItemExtensionBuilder},
    Extension, ExtensionMap,
};
use tracing::instrument;

use crate::{
    http_server::{
        errors::ServerError,
        pages::blog::md::html::{IntoHtml, MarkdownRenderContext},
        templates::{base_constrained, header::OpenGraph, ShortDesc},
        LinkTo, ResponseResult,
    },
    AppConfig, AppState,
};

pub(crate) struct PodcastEpisodeList<'a>(pub(crate) Vec<&'a PodcastEpisode>);

impl Render for PodcastEpisodeList<'_> {
    fn render(&self) -> Markup {
        html! {
            ul {
                @for ep in &self.0 {
                    li class="my-4" {
                        a href=(ep.relative_link()) {
                            span class="text-subtitle text-sm inline-block w-[80px]" {
                                (ep.frontmatter.date)
                            }
                            " "
                            (ep.frontmatter.title)
                        }
                    }
                }
            }
        }
    }
}

#[instrument(skip_all)]
pub(crate) async fn podcast_index(
    State(episodes): State<Arc<PodcastEpisodes>>,
) -> Result<Markup, StatusCode> {
    Ok(base_constrained(
        html! {
            h1 class="text-3xl" { "coreyja.fm Podcast" }
            p class="my-4 max-w-prose" {
                "Subscribe via "
                a href="/podcast/feed.xml" class="underline" { "RSS" }
            }
            (PodcastEpisodeList(episodes.by_recency()))
        },
        OpenGraph {
            title: "coreyja.fm Podcast".to_string(),
            description: Some("The coreyja.fm podcast".to_string()),
            ..Default::default()
        },
    ))
}

#[instrument(skip(episodes, state))]
pub(crate) async fn podcast_get(
    State(episodes): State<Arc<PodcastEpisodes>>,
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ResponseResult<Markup> {
    let ep = episodes
        .episodes
        .iter()
        .find(|e| e.frontmatter.slug == slug)
        .ok_or_else(|| {
            ServerError(
                cja::color_eyre::eyre::eyre!("No such podcast episode found"),
                StatusCode::NOT_FOUND,
            )
        })?;

    let markdown = ep.markdown();
    let context = MarkdownRenderContext {
        syntax_highlighting: state.syntax_highlighting_context.clone(),
        current_article_path: ep.relative_link(),
    };

    Ok(base_constrained(
        html! {
            h1 class="text-2xl" { (markdown.title) }
            subtitle class="block text-lg text-subtitle mb-8" { (markdown.date) }

            @if !ep.frontmatter.youtube_id.is_empty() {
                div class="my-4 aspect-video max-w-2xl" {
                    iframe
                        class="w-full h-full"
                        src=(format!("https://www.youtube.com/embed/{}", ep.frontmatter.youtube_id))
                        frameborder="0"
                        allowfullscreen
                        allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                        {}
                }
            }

            @if let Some(youtube_url) = &ep.frontmatter.youtube_url {
                p class="my-2" {
                    a href=(youtube_url) class="underline" { "Watch on YouTube" }
                }
            }

            div {
                (markdown.ast.into_html(&state.app, &context)?)
            }
        },
        OpenGraph {
            title: markdown.title.clone(),
            r#type: "article".to_string(),
            description: ep.short_description(),
            ..Default::default()
        },
    ))
}

#[instrument(skip_all)]
pub(crate) async fn podcast_rss_feed(
    State(state): State<AppState>,
    State(episodes): State<Arc<PodcastEpisodes>>,
) -> Result<impl IntoResponse, ServerError> {
    let channel = build_podcast_channel(&episodes, &state.app, &state.syntax_highlighting_context)?;

    let body = channel.to_string();
    let response = Response::builder()
        .header("Content-Type", "application/rss+xml")
        .body(body);

    match response {
        Ok(r) => Ok(r.into_response()),
        Err(_) => Err(cja::color_eyre::eyre::eyre!("Failed to build RSS Feed response").into()),
    }
}

fn build_podcast_channel(
    episodes: &PodcastEpisodes,
    config: &AppConfig,
    context: &crate::http_server::pages::blog::md::SyntaxHighlightingContext,
) -> cja::Result<rss::Channel> {
    let items: cja::Result<Vec<rss::Item>> = episodes
        .by_recency()
        .iter()
        .map(|ep| podcast_rss_item(ep, config, context))
        .collect();
    let items = items?;

    let itunes_ext = ITunesChannelExtensionBuilder::default()
        .author(Some("Corey Alexander".to_string()))
        .summary(Some("The coreyja.fm podcast".to_string()))
        .explicit(Some("no".to_string()))
        .build();

    let mut namespaces = BTreeMap::new();
    namespaces.insert(
        "podcast".to_string(),
        "https://podcastindex.org/namespace/1.0".to_string(),
    );

    Ok(rss::ChannelBuilder::default()
        .title("coreyja.fm".to_string())
        .link(config.app_url("/podcast"))
        .description("The coreyja.fm podcast".to_string())
        .copyright(Some("Copyright Corey Alexander".to_string()))
        .language(Some("en-us".to_string()))
        .itunes_ext(Some(itunes_ext))
        .namespaces(namespaces)
        .items(items)
        .build())
}

fn podcast_rss_item(
    ep: &PodcastEpisode,
    config: &AppConfig,
    context: &crate::http_server::pages::blog::md::SyntaxHighlightingContext,
) -> cja::Result<rss::Item> {
    let link = config.app_url(&format!("/podcast/{}", ep.frontmatter.slug));
    let posted_on = ep.frontmatter.date.and_time(NaiveTime::MIN).and_utc();

    let enclosure = rss::EnclosureBuilder::default()
        .url(ep.frontmatter.audio_url.clone())
        .length(ep.frontmatter.audio_length_bytes.to_string())
        .mime_type("audio/mpeg".to_string())
        .build();

    let itunes_ext = ITunesItemExtensionBuilder::default()
        .duration(Some(ep.frontmatter.audio_duration.clone()))
        .author(Some("Corey Alexander".to_string()))
        .build();

    let render_context = MarkdownRenderContext {
        syntax_highlighting: context.clone(),
        current_article_path: format!("/podcast/{}", ep.frontmatter.slug),
    };

    let guid = rss::GuidBuilder::default()
        .value(link.clone())
        .permalink(true)
        .build();

    let mut extensions: ExtensionMap = BTreeMap::new();
    if let Some(transcript_url) = &ep.frontmatter.transcript_url {
        let mut transcript_ext = Extension::default();
        transcript_ext.name = "podcast:transcript".to_string();
        transcript_ext
            .attrs
            .insert("url".to_string(), transcript_url.clone());
        transcript_ext
            .attrs
            .insert("type".to_string(), "application/srt".to_string());

        let mut podcast_exts: BTreeMap<String, Vec<Extension>> = BTreeMap::new();
        podcast_exts.insert("transcript".to_string(), vec![transcript_ext]);
        extensions.insert("podcast".to_string(), podcast_exts);
    }

    Ok(rss::ItemBuilder::default()
        .title(Some(ep.frontmatter.title.clone()))
        .link(Some(link))
        .guid(Some(guid))
        .description(ep.short_description())
        .pub_date(Some(posted_on.to_rfc2822()))
        .enclosure(Some(enclosure))
        .itunes_ext(Some(itunes_ext))
        .extensions(extensions)
        .content(Some(
            ep.markdown()
                .ast
                .0
                .into_html(config, &render_context)?
                .into_string(),
        ))
        .build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use posts::podcast::PodcastEpisodes;
    use url::Url;

    fn test_config() -> AppConfig {
        AppConfig {
            base_url: Url::parse("https://coreyja.com").unwrap(),
            imgproxy_url: None,
        }
    }

    fn test_context() -> crate::http_server::pages::blog::md::SyntaxHighlightingContext {
        crate::http_server::pages::blog::md::SyntaxHighlightingContext
    }

    #[test]
    fn test_rss_feed_is_valid_xml() {
        let episodes = PodcastEpisodes::from_static_dir().unwrap();
        let channel = build_podcast_channel(&episodes, &test_config(), &test_context()).unwrap();
        let xml = channel.to_string();

        let parsed: rss::Channel = xml.parse().expect("RSS feed should be valid XML");
        assert_eq!(parsed.title(), "coreyja.fm");
        assert!(!parsed.items().is_empty(), "Feed should have items");
    }

    #[test]
    fn test_rss_items_have_required_podcast_fields() {
        let episodes = PodcastEpisodes::from_static_dir().unwrap();
        let channel = build_podcast_channel(&episodes, &test_config(), &test_context()).unwrap();

        for item in channel.items() {
            assert!(item.title().is_some(), "Item must have a title");
            assert!(item.link().is_some(), "Item must have a link");
            assert!(item.guid().is_some(), "Item must have a GUID");
            assert!(item.pub_date().is_some(), "Item must have a pub_date");
            assert!(item.enclosure().is_some(), "Item must have an enclosure");

            let enclosure = item.enclosure().unwrap();
            assert!(!enclosure.url().is_empty(), "Enclosure must have a URL");
            assert_eq!(enclosure.mime_type(), "audio/mpeg");
            let length: u64 = enclosure
                .length()
                .parse()
                .expect("Enclosure length must be a number");
            assert!(length > 0, "Enclosure length must be positive");

            let itunes = item.itunes_ext().expect("Item must have iTunes extension");
            assert!(
                itunes.duration().is_some(),
                "iTunes extension must have duration"
            );
        }
    }

    #[test]
    fn test_rss_feed_has_podcast_namespace() {
        let episodes = PodcastEpisodes::from_static_dir().unwrap();
        let channel = build_podcast_channel(&episodes, &test_config(), &test_context()).unwrap();
        let xml = channel.to_string();

        assert!(
            xml.contains("xmlns:podcast=\"https://podcastindex.org/namespace/1.0\""),
            "Feed must declare the podcast namespace"
        );
    }

    #[test]
    fn test_rss_items_with_transcript_have_podcast_transcript_tag() {
        let episodes = PodcastEpisodes::from_static_dir().unwrap();
        let channel = build_podcast_channel(&episodes, &test_config(), &test_context()).unwrap();
        let xml = channel.to_string();

        let eps_with_transcripts: Vec<_> = episodes
            .episodes
            .iter()
            .filter(|ep| ep.frontmatter.transcript_url.is_some())
            .collect();

        assert!(
            !eps_with_transcripts.is_empty(),
            "At least one episode should have a transcript"
        );

        for ep in &eps_with_transcripts {
            let transcript_url = ep.frontmatter.transcript_url.as_ref().unwrap();
            assert!(
                xml.contains(&format!("url=\"{transcript_url}\"")),
                "Feed XML must contain podcast:transcript with URL for {}",
                ep.frontmatter.slug,
            );
        }

        assert!(
            xml.contains("type=\"application/srt\""),
            "Transcript tag must specify SRT MIME type"
        );
    }

    #[test]
    fn test_rss_channel_has_itunes_metadata() {
        let episodes = PodcastEpisodes::from_static_dir().unwrap();
        let channel = build_podcast_channel(&episodes, &test_config(), &test_context()).unwrap();

        let itunes = channel
            .itunes_ext()
            .expect("Channel must have iTunes extension");
        assert_eq!(itunes.author(), Some("Corey Alexander"));
        assert_eq!(itunes.explicit(), Some("no"));
    }
}
