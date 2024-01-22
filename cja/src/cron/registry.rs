use std::{collections::HashMap, future::Future, pin::Pin, time::Duration};

use miette::Diagnostic;
use tokio::time::Instant;
use tracing::error;

use crate::{app_state::AppState as AS, jobs::Job};

pub struct CronRegistry<AppState: AS> {
    pub(super) jobs: HashMap<&'static str, CronJob<AppState>>,
}

#[allow(clippy::type_complexity)]
pub(super) struct CronJob<AppState: AS> {
    name: &'static str,
    func: Box<
        dyn Fn(AppState, String) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>
            + Send
            + Sync
            + 'static,
    >,
    interval: Duration,
}

#[derive(Debug, thiserror::Error, Diagnostic)]
#[error("TickError: {0}")]
pub enum TickError {
    JobError(String),
    SqlxError(sqlx::Error),
}

impl<AppState: AS> CronJob<AppState> {
    #[tracing::instrument(
        name = "cron_job.tick",
        skip_all,
        fields(
            cron_job.name = self.name,
            cron_job.interval = ?self.interval
        )
    )]
    pub(crate) async fn tick(
        &self,
        app_state: AppState,
        last_enqueue_map: &mut HashMap<&str, Instant>,
    ) -> Result<(), TickError> {
        let last_enqueue = last_enqueue_map.get(self.name);
        let context = format!("Cron@{}", app_state.version());

        if let Some(last_enqueue) = last_enqueue {
            let elapsed = last_enqueue.elapsed();
            if elapsed > self.interval {
                tracing::info!(
                    task_name = self.name,
                    time_since_last_run =? elapsed,
                    "Enqueuing Task"
                );
                (self.func)(app_state, context)
                    .await
                    .map_err(TickError::JobError)?;
                last_enqueue_map.insert(self.name, Instant::now());
            }
        } else {
            tracing::info!(task_name = self.name, "Enqueuing Task for first time");
            (self.func)(app_state, context)
                .await
                .map_err(TickError::JobError)?;
            last_enqueue_map.insert(self.name, Instant::now());
        }

        Ok(())
    }
}

impl<AppState: AS> CronRegistry<AppState> {
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
        }
    }

    #[tracing::instrument(name = "cron.register", skip_all, fields(cron_job.name = name, cron_job.interval = ?interval))]
    pub fn register(
        &mut self,
        name: &'static str,
        interval: Duration,
        job: impl Fn(AppState, String) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>
            + Send
            + Sync
            + 'static,
    ) {
        let cron_job = CronJob {
            name,
            func: Box::new(job),
            interval,
        };
        self.jobs.insert(name, cron_job);
    }

    #[tracing::instrument(name = "cron.register_job", skip_all, fields(cron_job.name = J::NAME, cron_job.interval = ?interval))]
    pub fn register_job<J: Job<AppState>>(&mut self, job: J, interval: Duration) {
        self.register(J::NAME, interval, move |app_state, context| {
            let job = job.clone();
            Box::pin(async move {
                J::enqueue(job, app_state, context)
                    .await
                    .map_err(|_| "Failed to enqueue job".to_string())
            })
        });
    }
}

impl<AppState: AS> Default for CronRegistry<AppState> {
    fn default() -> Self {
        Self::new()
    }
}
