use cja::jobs::{registry::JobRegistry, worker::JobFromDB, Job};

use crate::state::AppState;

use self::{sponsors::RefreshSponsors, youtube_videos::RefreshVideos};

pub mod sponsors;
pub mod youtube_videos;

pub(crate) struct Jobs;

#[async_trait::async_trait]
impl JobRegistry<AppState> for Jobs {
    async fn run_job(&self, job: &JobFromDB, app_state: AppState) -> miette::Result<()> {
        let payload = job.payload.clone();

        match job.name.as_str() {
            "RefreshSponsors" => RefreshSponsors::run_from_value(payload, app_state).await,
            "RefreshVideos" => RefreshVideos::run_from_value(payload, app_state).await,
            _ => Err(miette::miette!("Unknown job type: {}", job.name)),
        }
    }
}
