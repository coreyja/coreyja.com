use cja::impl_job_registry;

use crate::state::AppState;

use self::{
    discord_message_processor::ProcessDiscordMessage,
    discord_thread_create_processor::ProcessDiscordThreadCreate,
    linear_webhook_processor::ProcessLinearWebhook, sponsors::RefreshSponsors,
    thread_processor::ProcessThreadStep, youtube_videos::RefreshVideos,
};

pub mod bytes_discord_posts;
pub mod discord_message_processor;
pub mod discord_thread_create_processor;
pub mod linear_webhook_processor;
pub mod refresh_discord;
pub mod refresh_linkedin_token;
pub mod sponsors;
pub mod thread_processor;
pub mod youtube_videos;

impl_job_registry!(
    AppState,
    RefreshSponsors,
    RefreshVideos,
    bytes_discord_posts::PostByteSubmission,
    refresh_discord::RefreshDiscordChannels,
    refresh_linkedin_token::RefreshLinkedInToken,
    ProcessThreadStep,
    ProcessDiscordMessage,
    ProcessDiscordThreadCreate,
    ProcessLinearWebhook
);
