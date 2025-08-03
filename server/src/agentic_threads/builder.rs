use color_eyre::eyre::{eyre, Result};
use db::agentic_threads::{Stitch, Thread, ThreadType};
use sqlx::PgPool;
use uuid::Uuid;

use crate::memory::MemoryManager;

pub struct ThreadBuilder {
    pool: PgPool,
    goal: String,
    thread_type: ThreadType,
    branching_stitch_id: Option<Uuid>,
}

impl ThreadBuilder {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            goal: String::new(),
            thread_type: ThreadType::Autonomous,
            branching_stitch_id: None,
        }
    }

    pub fn with_goal(mut self, goal: impl Into<String>) -> Self {
        self.goal = goal.into();
        self
    }

    pub fn with_thread_type(mut self, thread_type: ThreadType) -> Self {
        self.thread_type = thread_type;
        self
    }

    pub fn child_of(mut self, parent_stitch_id: Uuid) -> Self {
        self.branching_stitch_id = Some(parent_stitch_id);
        self
    }

    pub async fn build(self) -> Result<Thread> {
        // Validate
        if self.goal.is_empty() {
            return Err(eyre!("Thread goal cannot be empty"));
        }

        // Capture thread_type for later use
        let is_discord = self.thread_type == ThreadType::Interactive;

        // Create the thread using the unified method
        let thread = Thread::create(
            &self.pool,
            self.goal,
            self.branching_stitch_id,
            Some(self.thread_type),
        )
        .await?;

        let memory_manager = MemoryManager::new(self.pool.clone());
        let system_prompt = memory_manager.generate_system_prompt(is_discord).await?;

        // Create system prompt stitch
        Stitch::create_system_prompt(&self.pool, thread.thread_id, system_prompt).await?;

        Ok(thread)
    }
}
