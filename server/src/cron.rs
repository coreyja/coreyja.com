use std::time::Duration;

use cja::cron::{CronRegistry, Worker};

use crate::{
    jobs::{
        refresh_discord::RefreshDiscordChannels, sponsors::RefreshSponsors,
        youtube_videos::RefreshVideos,
    },
    state::AppState,
};

fn one_hour() -> Duration {
    Duration::from_hours(1)
}

pub(crate) fn cron_registry() -> CronRegistry<AppState> {
    let mut registry = CronRegistry::new();

    registry.register_job(RefreshSponsors, None, one_hour());
    registry.register_job(RefreshVideos, None, one_hour());
    registry.register_job(RefreshDiscordChannels, None, one_hour());

    registry
}

pub(crate) async fn run_cron(
    app_state: AppState,
    registry: CronRegistry<AppState>,
) -> cja::Result<()> {
    Worker::new_with_timezone(
        app_state,
        registry,
        cja::chrono_tz::US::Eastern,
        Duration::from_secs(10),
    )
    .run(cja::jobs::CancellationToken::new())
    .await?;

    Ok(())
}
