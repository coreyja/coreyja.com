use cja::jobs::Job;
use google_youtube3::{
    api::PlaylistItem, hyper::client::HttpConnector, hyper_rustls::HttpsConnector, YouTube,
};
use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    github::sponsors::set_last_refresh_at, google::get_valid_google_token, state::AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshVideos;

#[async_trait::async_trait]
impl Job<AppState> for RefreshVideos {
    const NAME: &'static str = "RefreshVideos";

    async fn run(&self, app_state: crate::AppState) -> miette::Result<()> {
        let access_token = get_valid_google_token(&app_state).await?;

        let hub = google_youtube3::YouTube::new(
            google_youtube3::hyper::Client::builder().build(
                google_youtube3::hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_or_http()
                    .enable_http1()
                    .enable_http2()
                    .build(),
            ),
            access_token,
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

        let first_page = hub
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

        set_last_refresh_at(&app_state, "youtube_videos").await?;

        assign_videos_to_playlists(&hub, &app_state).await?;

        Ok(())
    }
}

async fn assign_videos_to_playlists(
    hub: &YouTube<HttpsConnector<HttpConnector>>,
    app_state: &AppState,
) -> miette::Result<()> {
    let playlists = hub
        .playlists()
        .list(&vec!["snippet".to_owned()])
        .mine(true)
        .doit()
        .await
        .into_diagnostic()?;

    // for playlist in playlists.1.items.unwrap() {
    //     insert_playlist_page(playlist, app_state, hub).await?;
    // }

    let mut current_result = Some(playlists);
    while let Some(result) = current_result {
        if let Some(playlists) = result.1.items {
            for playlist in playlists {
                insert_playlist_page(playlist, app_state, hub).await?;
            }
        }

        current_result = if let Some(next_page_token) = result.1.next_page_token {
            Some(
                hub.playlists()
                    .list(&vec!["snippet".to_owned()])
                    .mine(true)
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

async fn insert_playlist_page(
    playlist: google_youtube3::api::Playlist,
    app_state: &AppState,
    hub: &YouTube<HttpsConnector<HttpConnector>>,
) -> Result<(), miette::ErrReport> {
    let playlist_id = playlist
        .id
        .ok_or_else(|| miette::miette!("No playlist ID found"))?;
    let snippet = playlist
        .snippet
        .ok_or_else(|| miette::miette!("No snippet found"))?;
    let youtube_playlist_id = sqlx::query!(
        "INSERT INTO YoutubePlaylists (
                    youtube_playlist_id,
                    external_youtube_playlist_id,
                    title,
                    description
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4
                ) ON CONFLICT (external_youtube_playlist_id) DO UPDATE SET
                    title = excluded.title,
                    description = excluded.description
                    RETURNING youtube_playlist_id",
        uuid::Uuid::new_v4(),
        playlist_id,
        snippet.title,
        snippet.description,
    )
    .fetch_one(&app_state.db)
    .await
    .into_diagnostic()?
    .youtube_playlist_id;
    let page = hub
        .playlist_items()
        .list(&vec!["snippet".to_owned(), "contentDetails".to_owned()])
        .playlist_id(&playlist_id)
        .doit()
        .await
        .into_diagnostic()?;
    let mut current_result = Some(page);
    while let Some(result) = current_result {
        let page = result.1.items;
        insert_playlist_items_page(&app_state.db, youtube_playlist_id, page).await?;

        current_result = if let Some(next_page_token) = result.1.next_page_token {
            Some(
                hub.playlist_items()
                    .list(&vec!["snippet".to_owned(), "contentDetails".to_owned()])
                    .playlist_id(&playlist_id)
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

async fn insert_playlist_items_page(
    db: &PgPool,
    youtube_playlist_id: Uuid,
    page: Option<Vec<PlaylistItem>>,
) -> miette::Result<()> {
    let Some(page) = page else { return Ok(()) };

    for item in page {
        let Some(ref content_details) = item.content_details else {
            return Err(miette::miette!(
                "No content details found for item {:?}",
                item
            ));
        };
        let external_video_id = content_details
            .video_id
            .as_ref()
            .ok_or_else(|| miette::miette!("No video ID found for item {:?}", item))?;

        let local_video_id = sqlx::query!(
            "
            SELECT youtube_video_id FROM YoutubeVideos WHERE external_youtube_id = $1
            ",
            external_video_id
        )
        .fetch_one(db)
        .await
        .into_diagnostic()?
        .youtube_video_id;

        sqlx::query!(
            "
    INSERT INTO YoutubeVideoPlaylists (
        youtube_video_playlist_id,
        youtube_playlist_id,
        youtube_video_id
    ) VALUES (
        $1,
        $2,
        $3
    ) ON CONFLICT (youtube_playlist_id, youtube_video_id) DO NOTHING",
            Uuid::new_v4(),
            youtube_playlist_id,
            local_video_id,
        )
        .execute(db)
        .await
        .into_diagnostic()?;
    }

    Ok(())
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
            .and_then(|thumbnails| thumbnails.high)
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
