use crate::app_state::{self};

use super::worker::JobFromDB;

#[async_trait::async_trait]
pub trait JobRegistry<AppState: app_state::AppState> {
    async fn run_job(&self, job: &JobFromDB, app_state: AppState) -> miette::Result<()>;
}
