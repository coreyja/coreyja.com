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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_name() {
        assert_eq!(RefreshSponsors::NAME, "RefreshSponsors");
    }

    #[test]
    fn test_job_serialization() {
        let job = RefreshSponsors;

        // Test that the job can be serialized/deserialized
        let serialized = serde_json::to_string(&job).unwrap();
        let deserialized: RefreshSponsors = serde_json::from_str(&serialized).unwrap();

        // Since RefreshSponsors has no fields, this just verifies the derive macros work
        assert_eq!(format!("{job:?}"), format!("{:?}", deserialized));
    }
}
