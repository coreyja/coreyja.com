use crate::app_state::{self};

use super::worker::JobFromDB;

#[async_trait::async_trait]
pub trait JobRegistry<AppState: app_state::AppState> {
    async fn run_job(&self, job: &JobFromDB, app_state: AppState) -> miette::Result<()>;
}

#[macro_export]
macro_rules! impl_job_registry {
    ($state:ty, $($job_type:ty),*) => {
        pub(crate) struct Jobs;

        #[async_trait::async_trait]
        impl JobRegistry<$state> for Jobs {
            async fn run_job(&self, job: &JobFromDB, app_state: $state) -> miette::Result<()> {
                let payload = job.payload.clone();

                match job.name.as_str() {
                    $(
                        <$job_type>::NAME => <$job_type>::run_from_value(payload, app_state).await,
                    )*
                    _ => Err(miette::miette!("Unknown job type: {}", job.name)),
                }
            }
        }
    };
}
