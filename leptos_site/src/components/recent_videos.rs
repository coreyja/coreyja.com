use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use leptos::*;
use leptos_query::{use_query, QueryResult, RefetchFn};
use leptos_router::A;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoutubeVideo {
    pub title: String,
    pub description: Option<String>,
    pub youtube_video_id: String,
    pub external_youtube_id: String,
    pub thumbnail_url: Option<String>,
    pub published_at: Option<chrono::NaiveDateTime>,
}

#[server]
pub async fn fetch_recent_videos(_args: ()) -> Result<Vec<YoutubeVideo>, ServerFnError> {
    use crate::server::extractors::extract_state;

    let state = extract_state()?;

    let recent_videos = sqlx::query_as!(
        YoutubeVideo,
        "SELECT *
      FROM YoutubeVideos
      ORDER BY published_at DESC LIMIT 3"
    )
    .fetch_all(&state.db)
    .await?;
    Ok(recent_videos)
}

fn use_fetch_recent_videos() -> QueryResult<Result<Vec<YoutubeVideo>, ServerFnError>, impl RefetchFn>
{
    use_query(|| (), fetch_recent_videos, Default::default())
}

#[component]
fn RecentVideo(video: YoutubeVideo) -> impl IntoView {
    let title1 = video.title.clone();
    let title2 = video.title.clone();
    view! {
        <li class="my-8">
            <A href=format!("/videos/{}", video.youtube_video_id)>
                <img
                    class="h-[180px] aspect-video object-cover object-center mb-2"
                    src=move || video.thumbnail_url.clone()
                    alt=title1
                    loading="lazy"
                />
            </A>
            <p class="max-w-[340px]">{title2}</p>

            <p class="text-subtitle text-sm">{video.published_at.map(|t| t.date().to_string())}</p>
        </li>
    }
}

#[component]
pub fn RecentVideos() -> impl IntoView {
    let QueryResult { data, .. } = use_fetch_recent_videos();
    let videos = move || data.get().map(|tils| tils.clone().unwrap_or_default());

    view! {
        <Suspense>
            {move || {
                videos()
                    .map(|videos| {
                        view! {
                            <ul class="flex flex-row flex-wrap">
                                <For
                                    each=move || videos.clone()
                                    key=move |data| data.youtube_video_id.clone()
                                    let:video
                                >
                                    <RecentVideo video=video.clone()/>
                                </For>
                            </ul>
                        }
                    })
            }}

        </Suspense>
    }
}
