use color_eyre::eyre::{eyre, Result};
use db::agentic_threads::{Stitch, Thread, ThreadType};
use db::discord_threads::DiscordThreadMetadata;
use db::linear_threads::LinearThreadMetadata;
use sqlx::PgPool;
use uuid::Uuid;

use crate::memory::MemoryManager;

pub struct DiscordMetadata {
    pub discord_thread_id: String,
    pub channel_id: String,
    pub guild_id: String,
    pub created_by: String,
    pub thread_name: String,
}

pub struct LinearMetadata {
    pub session_id: String,
    pub workspace_id: String,
    pub issue_id: Option<String>,
    pub issue_title: Option<String>,
    pub project_id: Option<String>,
    pub team_id: Option<String>,
    pub created_by_user_id: String,
}

pub struct ThreadBuilder {
    pool: PgPool,
    goal: String,
    thread_type: ThreadType,
    branching_stitch_id: Option<Uuid>,
    discord_metadata: Option<DiscordMetadata>,
    linear_metadata: Option<LinearMetadata>,
}

impl ThreadBuilder {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            goal: String::new(),
            thread_type: ThreadType::Autonomous,
            branching_stitch_id: None,
            discord_metadata: None,
            linear_metadata: None,
        }
    }

    pub fn with_goal(mut self, goal: impl Into<String>) -> Self {
        self.goal = goal.into();
        self
    }

    pub fn autonomous(mut self) -> Self {
        self.thread_type = ThreadType::Autonomous;
        self
    }

    pub fn interactive_discord(mut self, metadata: DiscordMetadata) -> Self {
        self.thread_type = ThreadType::Interactive;
        self.discord_metadata = Some(metadata);
        self
    }

    pub fn interactive_linear(mut self, metadata: LinearMetadata) -> Self {
        self.thread_type = ThreadType::Interactive;
        self.linear_metadata = Some(metadata);
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

        // Validate metadata if thread type is Interactive
        if self.thread_type == ThreadType::Interactive
            && self.discord_metadata.is_none()
            && self.linear_metadata.is_none()
        {
            return Err(eyre!(
                "Interactive threads must have either Discord or Linear metadata"
            ));
        }

        // Create the thread using the unified method
        let thread = Thread::create(
            &self.pool,
            self.goal,
            self.branching_stitch_id,
            Some(self.thread_type),
        )
        .await?;

        // Create Discord metadata if this is a Discord interactive thread
        if let Some(discord_meta) = self.discord_metadata {
            DiscordThreadMetadata::create(
                &self.pool,
                thread.thread_id,
                discord_meta.discord_thread_id,
                discord_meta.channel_id,
                discord_meta.guild_id,
                discord_meta.created_by,
                discord_meta.thread_name,
            )
            .await?;
        }

        // Create Linear metadata if this is a Linear interactive thread
        if let Some(linear_meta) = self.linear_metadata {
            LinearThreadMetadata::create(
                &self.pool,
                thread.thread_id,
                linear_meta.session_id,
                linear_meta.workspace_id,
                linear_meta.issue_id,
                linear_meta.issue_title,
                linear_meta.project_id,
                linear_meta.team_id,
                linear_meta.created_by_user_id,
            )
            .await?;
        }

        let memory_manager = MemoryManager::new(self.pool.clone());
        let system_prompt = memory_manager.generate_system_prompt(&thread).await?;

        // Create system prompt stitch
        Stitch::create_system_prompt(&self.pool, thread.thread_id, system_prompt).await?;

        Ok(thread)
    }
}
