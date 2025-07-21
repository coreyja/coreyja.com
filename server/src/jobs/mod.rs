use cja::impl_job_registry;

use crate::state::AppState;

use self::{
    discord_event_processor::ProcessDiscordEvent, sponsors::RefreshSponsors,
    thread_processor::ProcessThreadStep, youtube_videos::RefreshVideos,
};

pub mod bytes_discord_posts;
pub mod discord_event_processor;
pub mod refresh_discord;
pub mod sponsors;
pub mod standup_message;
pub mod thread_processor;
pub mod youtube_videos;

impl_job_registry!(
    AppState,
    RefreshSponsors,
    RefreshVideos,
    bytes_discord_posts::PostByteSubmission,
    refresh_discord::RefreshDiscordChannels,
    standup_message::StandupMessage,
    ProcessThreadStep,
    ProcessDiscordEvent
);
