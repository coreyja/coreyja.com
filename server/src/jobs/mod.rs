use serde::{de::DeserializeOwned, Serialize};

use crate::AppState;
use miette::IntoDiagnostic;

mod sponsors;

#[async_trait::async_trait]
trait Job: Serialize + DeserializeOwned + Send + Sync + std::fmt::Debug + Clone + 'static {
    const NAME: &'static str;

    async fn run(&self, app_state: AppState) -> miette::Result<()>;

    async fn enqueue(self, app_state: AppState) -> miette::Result<()> {
        sqlx::query!(
            "
        INSERT INTO jobs (name, payload, priority, run_at, created_at, context)
        VALUES ($1, $2, $3, $4, $5, $6)",
            Self::NAME,
            serde_json::to_value(self).into_diagnostic()?,
            0,
            chrono::Utc::now(),
            chrono::Utc::now(),
            ""
        )
        .execute(&app_state.db)
        .await
        .into_diagnostic()?;

        Ok(())
    }
}
