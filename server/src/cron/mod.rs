use std::{collections::HashMap, time::Duration};

use miette::Result;
use tokio::time::{sleep, Instant};

use crate::{
    jobs::{sponsors::RefreshSponsors, Job},
    AppState,
};

fn one_hour() -> Duration {
    Duration::from_secs(60 * 60)
}

pub async fn run_cron(app_state: AppState) -> Result<()> {
    let mut last_enqueue_map: HashMap<&str, Instant> = HashMap::new();

    tracing::debug!("Starting cron loop");
    loop {
        tracing::debug!("Cron Loop Tick");
        let refresh = last_enqueue_map.get("RefreshSponsors");
        if let Some(last_enqueue) = refresh {
            let elapsed = last_enqueue.elapsed();
            if elapsed > one_hour() {
                tracing::info!(
                    task_name = "RefreshSponsors",
                    time_since_last_run =? elapsed,
                    "Enqueuing Task"
                );
                RefreshSponsors.enqueue(app_state.clone()).await?;
                last_enqueue_map.insert("RefreshSponsors", Instant::now());
            }
        } else {
            tracing::info!(
                task_name = "RefreshSponsors",
                "Enqueuing Task for first time"
            );
            RefreshSponsors.enqueue(app_state.clone()).await?;
            last_enqueue_map.insert("RefreshSponsors", Instant::now());
        }

        sleep(Duration::from_secs(2)).await;
    }
}
