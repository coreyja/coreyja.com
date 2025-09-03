use std::time::Duration;

use cja::cron::{CronRegistry, Worker};

use crate::{
    jobs::{
        refresh_discord::RefreshDiscordChannels, sponsors::RefreshSponsors,
        standup_message::StandupMessage, youtube_videos::RefreshVideos,
    },
    state::AppState,
};

fn one_hour() -> Duration {
    Duration::from_secs(60 * 60)
}

pub(crate) fn cron_registry() -> cja::Result<CronRegistry<AppState>> {
    let mut registry = CronRegistry::new();

    registry.register_job(RefreshSponsors, one_hour());
    registry.register_job(RefreshVideos, one_hour());
    registry.register_job(RefreshDiscordChannels, one_hour());

    registry.register_job_with_cron(StandupMessage, "0 10 7 * * * *")?;

    Ok(registry)
}

pub(crate) async fn run_cron(app_state: AppState) -> cja::Result<()> {
    Worker::new_with_timezone(
        app_state,
        cron_registry()?,
        cja::chrono_tz::US::Eastern,
        Duration::from_secs(10),
    )
    .run()
    .await?;

    Ok(())
}
