use std::time::Duration;

use cja::cron::{CronRegistry, Worker};

use crate::{
    jobs::{sponsors::RefreshSponsors, youtube_videos::RefreshVideos},
    state::AppState,
};

fn one_hour() -> Duration {
    Duration::from_secs(60 * 60)
}

fn cron_registry() -> CronRegistry<AppState> {
    let mut registry = CronRegistry::new();

    registry.register_job(RefreshSponsors, one_hour());
    registry.register_job(RefreshVideos, one_hour());

    registry
}

pub(crate) async fn run_cron(app_state: AppState) -> cja::Result<()> {
    Worker::new(app_state, cron_registry()).run().await?;

    Ok(())
}
