use std::{collections::HashMap, time::Duration};

use chrono::{DateTime, Utc};

use crate::app_state::AppState as AS;

use super::registry::{CronRegistry, TickError};

pub struct Worker<AppState: AS> {
    state: AppState,
    registry: CronRegistry<AppState>,
}

impl<AppState: AS> Worker<AppState> {
    pub fn new(state: AppState, registry: CronRegistry<AppState>) -> Self {
        Self { state, registry }
    }

    pub async fn run(self) -> Result<(), TickError> {
        let worker_id = uuid::Uuid::new_v4();

        tracing::debug!("Starting cron loop");
        loop {
            let last_runs = sqlx::query!(
                "SELECT name, max(last_run_at) as last_run_at FROM Crons GROUP BY name"
            )
            .fetch_all(self.state.db())
            .await
            .map_err(TickError::SqlxError)?;

            let last_run_map: HashMap<&str, DateTime<Utc>> = last_runs
                .iter()
                .map(|row| (row.name.as_str(), row.last_run_at.unwrap_or_default()))
                .collect();
            self.tick(&worker_id, &last_run_map).await?;

            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }

    #[tracing::instrument(name = "cron.tick", skip_all, fields(cron_worker.id = %worker_id))]
    async fn tick(
        &self,
        worker_id: &uuid::Uuid,
        last_enqueue_map: &HashMap<&str, chrono::DateTime<Utc>>,
    ) -> Result<(), TickError> {
        for job in self.registry.jobs.values() {
            job.tick(self.state.clone(), last_enqueue_map).await?;
        }

        Ok(())
    }
}
