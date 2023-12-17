use google_youtube3::api::PlaylistItem;
use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::Job;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshVideos;

#[async_trait::async_trait]
impl Job for RefreshVideos {
    const NAME: &'static str = "RefreshVideos";

    async fn run(&self, app_state: crate::AppState) -> miette::Result<()> {
        let hub = google_youtube3::YouTube::new(
            google_youtube3::hyper::Client::builder().build(
                google_youtube3::hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_or_http()
                    .enable_http1()
                    .enable_http2()
                    .build(),
            ),
            std::env::var("COREYJA_YOUTUBE_ACCESS_TOKEN")
                .unwrap()
                .to_string(),
        );

        let channels = hub
            .channels()
            .list(&vec!["contentDetails".to_owned(), "statistics".to_owned()])
            .mine(true)
            .doit()
            .await
            .into_diagnostic()?
            .1;

        let upload_playlist = channels
            .items
            .ok_or_else(|| miette::miette!("No items in channels response"))?
            .into_iter()
            .next()
            .ok_or_else(|| miette::miette!("No channels marked as mine found"))?
            .content_details
            .ok_or_else(|| {
                miette::miette!("No content details found, is the part missing from the request?")
            })?
            .related_playlists
            .ok_or_else(|| {
                miette::miette!(
                    "No related playlists found, is the body format different/malformed"
                )
            })?
            .uploads
            .ok_or_else(|| miette::miette!("No uploads playlist found"))?;

        let mut first_page = hub
            .playlist_items()
            .list(&vec!["snippet".to_owned(), "contentDetails".to_owned()])
            .playlist_id(&upload_playlist)
            .doit()
            .await
            .into_diagnostic()?;

        let mut current_result = Some(first_page);

        while let Some(result) = current_result {
            let page = result.1.items;
            insert_youtube_video_page(&app_state.db, page).await?;

            current_result = if let Some(next_page_token) = result.1.next_page_token {
                Some(
                    hub.playlist_items()
                        .list(&vec!["snippet".to_owned(), "contentDetails".to_owned()])
                        .playlist_id(&upload_playlist)
                        .page_token(&next_page_token)
                        .doit()
                        .await
                        .into_diagnostic()?,
                )
            } else {
                None
            };
        }

        Ok(())
    }
}

async fn insert_youtube_video_page(
    db: &PgPool,
    page: Option<Vec<PlaylistItem>>,
) -> miette::Result<()> {
    let Some(page) = page else { return Ok(()) };

    for item in page {
        let Some(content_details) = item.content_details else {
            return Err(miette::miette!(
                "No content details found for item {:?}",
                item
            ));
        };

        let video_id = content_details
            .video_id
            .ok_or_else(|| miette::miette!("No video ID found for item"))?;

        let snippet = item
            .snippet
            .ok_or_else(|| miette::miette!("No snippet found for item"))?;

        let title = snippet
            .title
            .ok_or_else(|| miette::miette!("No title found for item"))?;

        let description = snippet
            .description
            .ok_or_else(|| miette::miette!("No description found for item"))?;

        let published_at = snippet
            .published_at
            .ok_or_else(|| miette::miette!("No published_at found for item"))?;

        let thumbnail_url = snippet
            .thumbnails
            .and_then(|thumbnails| thumbnails.default)
            .and_then(|thumbnail| thumbnail.url)
            .ok_or_else(|| miette::miette!("No thumbnail_url found for item"))?;

        sqlx::query!(
            "
            INSERT INTO YoutubeVideos (
                youtube_video_id,
                external_youtube_id,
                title,
                description,
                published_at,
                thumbnail_url
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (external_youtube_id) DO UPDATE SET
                title = $3,
                description = $4,
                published_at = $5,
                thumbnail_url = $6
            RETURNING *
            ",
            uuid::Uuid::new_v4(),
            video_id,
            title,
            description,
            published_at.naive_utc(),
            thumbnail_url,
        )
        .fetch_one(db)
        .await
        .into_diagnostic()?;
    }

    Ok(())
}
