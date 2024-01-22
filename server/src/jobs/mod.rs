use cja::{
    impl_job_registry,
    jobs::{registry::JobRegistry, worker::JobFromDB, Job},
};

use crate::state::AppState;

use self::{sponsors::RefreshSponsors, youtube_videos::RefreshVideos};

pub mod sponsors;
pub mod youtube_videos;

impl_job_registry!(AppState, RefreshSponsors, RefreshVideos);
