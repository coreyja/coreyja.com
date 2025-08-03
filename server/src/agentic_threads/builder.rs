use color_eyre::eyre::{eyre, Result};
use db::agentic_threads::{Stitch, Thread, ThreadType};
use db::discord_threads::DiscordThreadMetadata;
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

pub struct ThreadBuilder {
    pool: PgPool,
    goal: String,
    thread_type: ThreadType,
    branching_stitch_id: Option<Uuid>,
    discord_metadata: Option<DiscordMetadata>,
}

impl ThreadBuilder {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            goal: String::new(),
            thread_type: ThreadType::Autonomous,
            branching_stitch_id: None,
            discord_metadata: None,
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

    pub fn child_of(mut self, parent_stitch_id: Uuid) -> Self {
        self.branching_stitch_id = Some(parent_stitch_id);
        self
    }

    pub async fn build(self) -> Result<Thread> {
        // Validate
        if self.goal.is_empty() {
            return Err(eyre!("Thread goal cannot be empty"));
        }

        // Validate Discord metadata if thread type is Interactive
        if self.thread_type == ThreadType::Interactive && self.discord_metadata.is_none() {
            return Err(eyre!(
                "Interactive threads must have Discord metadata. This should"
            ));
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

        // Create Discord metadata if this is an interactive thread
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

        let memory_manager = MemoryManager::new(self.pool.clone());
        let system_prompt = memory_manager.generate_system_prompt(is_discord).await?;

        // Create system prompt stitch
        Stitch::create_system_prompt(&self.pool, thread.thread_id, system_prompt).await?;

        Ok(thread)
    }
}
