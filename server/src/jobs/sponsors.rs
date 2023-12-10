use serde::{Deserialize, Serialize};

use crate::github::sponsors::refresh_db;

use super::Job;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RefreshSponsors;

#[async_trait::async_trait]
impl Job for RefreshSponsors {
    const NAME: &'static str = "RefreshSponsors";

    async fn run(&self, app_state: crate::AppState) -> miette::Result<()> {
        refresh_db(&app_state).await?;

        Ok(())
    }
}
