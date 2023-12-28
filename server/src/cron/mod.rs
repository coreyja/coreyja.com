use std::time::Duration;

use miette::Result;

use crate::{
    jobs::{sponsors::RefreshSponsors, youtube_videos::RefreshVideos, Job},
    AppState,
};

mod registry;
use registry::CronRegistry;

mod worker;

fn one_hour() -> Duration {
    Duration::from_secs(60 * 60)
}

pub(crate) fn cron_registry() -> CronRegistry {
    let mut registry = CronRegistry::new();

    registry.register("RefreshSponsors", one_hour(), |app_state, context| {
        RefreshSponsors.enqueue(app_state, context)
    });

    registry.register("RefreshVideos", one_hour(), |app_state, context| {
        RefreshVideos.enqueue(app_state, context)
    });

    registry
}

pub(crate) async fn run_cron(app_state: AppState) -> Result<()> {
    let worker = worker::Worker::new(app_state, cron_registry());

    worker.run().await
}
