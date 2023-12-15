use std::time::Duration;

use miette::Result;

use crate::{
    jobs::{sponsors::RefreshSponsors, Job},
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

    registry.run().await
}
