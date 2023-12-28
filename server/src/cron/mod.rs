use std::time::Duration;

use miette::Result;

use crate::{
    http_server::admin::job_routes::refresh_youtube,
    jobs::{sponsors::RefreshSponsors, youtube_videos::RefreshVideos, Job},
    AppState,
};

mod registry;
use registry::CronRegistry;

fn one_hour() -> Duration {
    Duration::from_secs(60 * 60)
}

pub(crate) async fn run_cron(app_state: AppState) -> Result<()> {
    let mut registry = CronRegistry::new(app_state);

    registry.register("RefreshSponsors", one_hour(), |app_state| {
        RefreshSponsors.enqueue(app_state)
    });

    registry.register("RefreshVideos", one_hour(), |app_state| {
        RefreshVideos.enqueue(app_state)
    });

    registry.run().await
}
