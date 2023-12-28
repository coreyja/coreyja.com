use std::{collections::HashMap, time::Duration};

use miette::Result;
use tokio::time::Instant;

use crate::state::AppState;

use super::registry::CronRegistry;

pub(super) struct Worker {
    state: AppState,
    registry: CronRegistry,
}

impl Worker {
    pub fn new(state: AppState, registry: CronRegistry) -> Self {
        Self { state, registry }
    }

    pub async fn run(self) -> Result<()> {
        let worker_id = uuid::Uuid::new_v4();
        let mut last_enqueue_map: HashMap<&str, Instant> = HashMap::new();

        tracing::debug!("Starting cron loop");
        loop {
            self.tick(&worker_id, &mut last_enqueue_map).await?;

            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }

    #[tracing::instrument(name = "cron.tick", skip_all, fields(cron_worker.id = %worker_id))]
    async fn tick(
        &self,
        worker_id: &uuid::Uuid,
        last_enqueue_map: &mut HashMap<&str, Instant>,
    ) -> Result<(), miette::ErrReport> {
        for (_, job) in self.registry.jobs.iter() {
            job.tick(self.state.clone(), last_enqueue_map).await?;
        }

        Ok(())
    }
}
