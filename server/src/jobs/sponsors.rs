use cja::jobs::Job;
use serde::{Deserialize, Serialize};

use crate::{github::sponsors::refresh_db, state::AppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshSponsors;

#[async_trait::async_trait]
impl Job<AppState> for RefreshSponsors {
    const NAME: &'static str = "RefreshSponsors";

    async fn run(&self, app_state: crate::AppState) -> cja::Result<()> {
        refresh_db(&app_state).await?;

        Ok(())
    }
}
