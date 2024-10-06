use cja::impl_job_registry;

use crate::state::AppState;

use self::{sponsors::RefreshSponsors, youtube_videos::RefreshVideos};

pub mod bytes_discord_posts;
pub mod sponsors;
pub mod youtube_videos;

impl_job_registry!(
    AppState,
    RefreshSponsors,
    RefreshVideos,
    bytes_discord_posts::PostByteSubmission
);
