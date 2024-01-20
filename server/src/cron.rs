use std::time::Duration;

use cja::{
    cron::{CronRegistry, Worker},
    jobs::Job,
};

use crate::{
    jobs::{sponsors::RefreshSponsors, youtube_videos::RefreshVideos},
    state::AppState,
};

fn one_hour() -> Duration {
    Duration::from_secs(60 * 60)
}

fn cron_registry() -> CronRegistry<AppState> {
    let mut registry = CronRegistry::new();

    registry.register("RefreshSponsors", one_hour(), |app_state, context| {
        RefreshSponsors.enqueue(app_state, context)
    });

    registry.register("RefreshVideos", one_hour(), |app_state, context| {
        RefreshVideos.enqueue(app_state, context)
    });

    registry
}

pub(crate) async fn run_cron(app_state: AppState) -> miette::Result<()> {
    Worker::new(app_state, cron_registry()).run().await?;

    Ok(())
}
