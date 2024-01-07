use std::{collections::HashMap, future::Future, pin::Pin, time::Duration};

use miette::Diagnostic;
use tokio::time::Instant;
use tracing::error;

use crate::app_state::AppState as AS;

pub struct CronRegistry<AppState: AS> {
    pub(crate) jobs: HashMap<&'static str, CronJob<AppState>>,
}

#[async_trait::async_trait]
pub(super) trait CronFn<AppState: AS + 'static>
{
    // This collapses the error type to a string, right now thats because thats
    // what the only consumer really needs. As we add more error debugging we'll
    // need to change this.
    async fn run(
        &self,
        app_state: AppState,
        context: String,
    ) -> Result<(), String>;
}

#[async_trait::async_trait]
impl<AppState: AS + 'static, Func, FnError: Diagnostic> CronFn<AppState> for Func where
    Func: Fn(AppState, String) -> Pin<Box<dyn Future<Output = Result<(), FnError>> + Send>> + Send + Sync
{
    async fn run(
        &self,
        app_state: AppState,
        context: String,
    ) -> Result<(), String>  {
        self(app_state, context).await.map_err(|err| format!("{err}"))
    }
}

pub(super) struct CronJob<AppState: AS> {
    name: &'static str,
    func: Box<dyn CronFn<AppState>>,
    interval: Duration,
}

trait CronJobTrait<AppState: AS, Error: Diagnostic> {
    fn run(
        &self,
        app_state: AppState,
    ) -> Result<(), TickError>;
}

#[derive(Debug, thiserror::Error, Diagnostic)]
#[error("TickError: {0}")]
pub(super) enum TickError {
    JobError(String),
    SqlxError(sqlx::Error),
}

impl<AppState: AS + 'static> CronJob<AppState> {
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
                (self.func).run(app_state, context).await.map_err(TickError::JobError)?;
                last_enqueue_map.insert(self.name, Instant::now());
            }
        } else {
            tracing::info!(task_name = self.name, "Enqueuing Task for first time");
            (self.func).run(app_state, context).await.map_err(TickError::JobError)?;
            last_enqueue_map.insert(self.name, Instant::now());
        }

        Ok(())
    }
}

impl<AppState: AS + 'static> CronRegistry<AppState> {
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
        }
    }

    #[tracing::instrument(name = "cron.register", skip_all, fields(cron_job.name = name, cron_job.interval = ?interval))]
    pub fn register<Error: Diagnostic>(&mut self, name: &'static str, interval: Duration, job: impl CronFn<AppState>) {
        let cron_job = CronJob {
            name,
            func: Box::new(job),
            interval,
        };
        self.jobs.insert(name, cron_job);
    }
}
