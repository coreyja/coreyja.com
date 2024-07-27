use std::{collections::HashMap, error::Error, future::Future, pin::Pin, time::Duration};

use chrono::{OutOfRangeError, Utc};
use tracing::error;

use crate::{app_state::AppState as AS, jobs::Job};

pub struct CronRegistry<AppState: AS> {
    pub(super) jobs: HashMap<&'static str, CronJob<AppState>>,
}

#[async_trait::async_trait]
pub trait CronFn<AppState: AS> {
    // This collapses the error type to a string, right now thats because thats
    // what the only consumer really needs. As we add more error debugging we'll
    // need to change this.
    async fn run(&self, app_state: AppState, context: String) -> Result<(), String>;
}

pub struct CronFnClosure<
    AppState: AS,
    FnError: Error + Send + Sync + 'static,
    F: Fn(AppState, String) -> Pin<Box<dyn Future<Output = Result<(), FnError>> + Send>>
        + Send
        + Sync
        + 'static,
> {
    pub(super) func: F,
    _marker: std::marker::PhantomData<AppState>,
}

#[async_trait::async_trait]
impl<
        AppState: AS,
        FnError: Error + Send + Sync + 'static,
        F: Fn(AppState, String) -> Pin<Box<dyn Future<Output = Result<(), FnError>> + Send>>
            + Send
            + Sync
            + 'static,
    > CronFn<AppState> for CronFnClosure<AppState, FnError, F>
{
    async fn run(&self, app_state: AppState, context: String) -> Result<(), String> {
        (self.func)(app_state, context)
            .await
            .map_err(|err| format!("{err:?}"))
    }
}

#[allow(clippy::type_complexity)]
pub(super) struct CronJob<AppState: AS> {
    name: &'static str,
    func: Box<dyn CronFn<AppState> + Send + Sync + 'static>,
    interval: Duration,
}

#[derive(Debug, thiserror::Error)]
#[error("TickError: {0}")]
pub enum TickError {
    JobError(String),
    SqlxError(sqlx::Error),
    NegativeDuration(OutOfRangeError),
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
        last_enqueue_map: &HashMap<&str, chrono::DateTime<Utc>>,
    ) -> Result<(), TickError> {
        let last_enqueue = last_enqueue_map.get(self.name);
        let context = format!("Cron@{}", app_state.version());
        let now = Utc::now();

        if let Some(last_enqueue) = last_enqueue {
            let elapsed = now - last_enqueue;
            let elapsed = elapsed.to_std().map_err(TickError::NegativeDuration)?;
            if elapsed > self.interval {
                tracing::info!(
                    task_name = self.name,
                    time_since_last_run =? elapsed,
                    "Enqueuing Task"
                );
                (self.func)
                    .run(app_state.clone(), context)
                    .await
                    .map_err(TickError::JobError)?;
            }
        } else {
            tracing::info!(task_name = self.name, "Enqueuing Task for first time");
            (self.func)
                .run(app_state.clone(), context)
                .await
                .map_err(TickError::JobError)?;
        }
        sqlx::query!(
            "INSERT INTO Crons (cron_id, name, last_run_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (name)
            DO UPDATE SET
            last_run_at = $3",
            uuid::Uuid::new_v4(),
            self.name,
            now,
            now,
            now
        )
        .execute(app_state.db())
        .await
        .map_err(TickError::SqlxError)?;

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
    pub fn register<FnError: Error + Send + Sync + 'static>(
        &mut self,
        name: &'static str,
        interval: Duration,
        job: impl Fn(AppState, String) -> Pin<Box<dyn Future<Output = Result<(), FnError>> + Send>>
            + Send
            + Sync
            + 'static,
    ) {
        let cron_job = CronJob {
            name,
            func: Box::new(CronFnClosure {
                func: job,
                _marker: std::marker::PhantomData,
            }),
            interval,
        };
        self.jobs.insert(name, cron_job);
    }

    #[tracing::instrument(name = "cron.register_job", skip_all, fields(cron_job.name = J::NAME, cron_job.interval = ?interval))]
    pub fn register_job<J: Job<AppState>>(&mut self, job: J, interval: Duration) {
        self.register(J::NAME, interval, move |app_state, context| {
            J::enqueue(job.clone(), app_state, context)
        });
    }
}

impl<AppState: AS> Default for CronRegistry<AppState> {
    fn default() -> Self {
        Self::new()
    }
}
