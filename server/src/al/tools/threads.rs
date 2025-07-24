use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    al::tools::{ThreadContext, Tool},
    AppState,
};

#[derive(Clone)]
pub struct CompleteThread;

impl CompleteThread {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CompleteThreadInput {
    pub reason: String,
}

#[async_trait::async_trait]
impl Tool for CompleteThread {
    const NAME: &'static str = "complete_thread";
    const DESCRIPTION: &'static str =
        "Mark the current thread as completed. This will end the conversation and mark the thread status as 'completed'. Use this tool when you've finished your work";

    type ToolInput = CompleteThreadInput;
    type ToolOutput = ();

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        use db::agentic_threads::Thread;
        use serde_json::json;

        // Mark the thread as complete with the reason
        Thread::complete(
            &app_state.db,
            context.thread.thread_id,
            json!({
                "reason": input.reason,
                "completed_at": chrono::Utc::now().to_rfc3339()
            }),
        )
        .await?;

        Ok(())
    }
}
