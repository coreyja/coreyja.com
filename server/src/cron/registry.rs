use std::{collections::HashMap, future::Future, pin::Pin, time::Duration};

use miette::Result;
use tokio::time::{sleep, Instant};

use crate::AppState;

pub(crate) struct CronRegistry {
    app_state: AppState,
    jobs: HashMap<&'static str, CronJob>,
}

pub(crate) trait CronFn:
    Fn(AppState) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync
{
}
impl<T> CronFn for T where
    T: Fn(AppState) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync
{
}

struct CronJob {
    name: &'static str,
    func: Box<dyn CronFn>,
    interval: Duration,
}

impl CronJob {
    async fn tick(
        &self,
        app_state: AppState,
        last_enqueue_map: &mut HashMap<&str, Instant>,
    ) -> Result<()> {
        let last_enqueue = last_enqueue_map.get(self.name);

        if let Some(last_enqueue) = last_enqueue {
            let elapsed = last_enqueue.elapsed();
            if elapsed > self.interval {
                tracing::info!(
                    task_name = self.name,
                    time_since_last_run =? elapsed,
                    "Enqueuing Task"
                );
                (self.func)(app_state).await?;
                last_enqueue_map.insert(self.name, Instant::now());
            }
        } else {
            tracing::info!(task_name = self.name, "Enqueuing Task for first time");
            (self.func)(app_state).await?;
            last_enqueue_map.insert(self.name, Instant::now());
        }

        Ok(())
    }
}

impl CronRegistry {
    pub fn new(app_state: AppState) -> Self {
        Self {
            app_state,
            jobs: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &'static str, duration: Duration, job: impl CronFn + 'static) {
        let cron_job = CronJob {
            name,
            func: Box::new(job),
            interval: duration,
        };
        self.jobs.insert(name, cron_job);
    }

    pub async fn run(self) -> Result<()> {
        let mut last_enqueue_map: HashMap<&str, Instant> = HashMap::new();

        tracing::debug!("Starting cron loop");
        loop {
            tracing::debug!("Cron Loop Tick");

            for (_, job) in self.jobs.iter() {
                job.tick(self.app_state.clone(), &mut last_enqueue_map)
                    .await?;
            }

            sleep(Duration::from_secs(2)).await;
        }
    }
}
