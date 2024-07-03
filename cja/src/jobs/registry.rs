use crate::app_state::{self};

use super::worker::JobFromDB;

#[async_trait::async_trait]
pub trait JobRegistry<AppState: app_state::AppState> {
    async fn run_job(&self, job: &JobFromDB, app_state: AppState) -> color_eyre::Result<()>;
}

#[macro_export]
macro_rules! impl_job_registry {
    ($state:ty, $($job_type:ty),*) => {
        pub(crate) struct Jobs;

        #[async_trait::async_trait]
        impl $crate::jobs::registry::JobRegistry<$state> for Jobs {
            async fn run_job(&self, job: &$crate::jobs::worker::JobFromDB, app_state: $state) -> $crate::Result<()> {
                use $crate::jobs::Job as _;

                let payload = job.payload.clone();

                match job.name.as_str() {
                    $(
                        <$job_type>::NAME => <$job_type>::run_from_value(payload, app_state).await,
                    )*
                    _ => Err($crate::color_eyre::eyre::eyre!("Unknown job type: {}", job.name)),
                }
            }
        }
    };
}
