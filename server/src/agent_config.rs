//! Agent Configuration System
//!
//! This module defines the Agent Configuration System which allows you to configure multiple AI
//! agents with different capabilities, tools, and behaviors. Each thread is assigned to an agent
//! that controls its available tools, persona, and behavior.
//!
//! ## Architecture
//!
//! ### Agent Configuration
//!
//! Agents are defined as variants in the [`AgentId`] enum. Each agent variant has an associated
//! configuration that defines:
//! - Agent name and description
//! - Discord channel and user IDs
//! - Persona identifier
//! - List of enabled tools
//!
//! The system uses an enum-based approach rather than a registry pattern, making it impossible to
//! forget to configure a new agent - if you add an enum variant, you must implement its `config()` method.
//!
//! ### Agent Structure
//!
//! Each agent has the following configuration via [`AgentConfig`]:
//!
//! - `id`: Unique [`AgentId`] enum variant for the agent (e.g., `AgentId::Al`, `AgentId::Demo`)
//! - `description`: Human-readable description
//! - `discord_channel_id`: Discord channel ID where this agent operates (for autonomous messages)
//! - `discord_user_id`: Discord user ID for the bot when posting
//! - `persona`: Persona identifier from `memory_blocks` table
//! - `enabled_tools`: List of enabled tool names
//!
//! ## Default Agent: "Al"
//!
//! The default agent is "Al" (defined as [`DEFAULT_AGENT_ID`]). This agent is used when:
//! - No specific agent is specified in thread creation
//! - An unknown agent name is referenced
//! - Creating threads via the API without specifying an agent
//!
//! ## Configuring Agents
//!
//! ### Adding a New Agent
//!
//! 1. Add a new variant to the [`AgentId`] enum:
//!
//! ```rust,ignore
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Display, EnumString)]
//! #[strum(serialize_all = "PascalCase")]
//! pub enum AgentId {
//!     Al,
//!     Demo,
//!     MyAgent,  // Add your new agent here
//! }
//! ```
//!
//! 2. Add a match arm in the `config()` method:
//!
//! ```rust,ignore
//! AgentId::MyAgent => AgentConfig {
//!     id: AgentId::MyAgent,
//!     description: "Description of what this agent does".to_string(),
//!     discord_channel_id: Some(1234567890),  // Optional
//!     discord_user_id: Some(9876543210),     // Optional
//!     persona: "agent-persona".to_string(),
//!     enabled_tools: vec![
//!         Tool::SendDiscordThreadMessage,
//!         Tool::ListenToThread,
//!         // Add other tools as needed
//!     ],
//! }
//! ```
//!
//! ### Configuration Location
//!
//! Agent configuration is code-based and defined in this module. All channel IDs, user IDs, and
//! other settings are hardcoded in the `AgentId::config()` method for easy management and version control.
//!
//! ## Tool Filtering
//!
//! Each agent has a whitelist of enabled tools. When a thread is processed, the system:
//!
//! 1. Parses the `thread.agent_name` string into an [`AgentId`] enum variant
//! 2. Gets the agent's configuration via `agent_id.config()`
//! 3. Checks if each tool is in the agent's `enabled_tools` list
//! 4. Only adds tools that are enabled for that agent
//!
//! If the agent name string cannot be parsed into a valid [`AgentId`], **all tools are enabled**
//! as a fallback for backward compatibility.
//!
//! ### Available Tools
//!
//! See the [`Tool`] enum for all available tools:
//! - Discord tools: `SendDiscordThreadMessage`, `ListenToThread`, `ReactToMessage`, etc.
//! - Linear tools: `ExecuteLinearQuery`, `SearchLinearQueries`, etc.
//! - Cooking tools: `UpsertRecipe`, `GetRecipeByName`, etc.
//! - Other tools: `SuggestionsSubmit`, `GetTags`, etc.
//!
//! ## Thread Assignment
//!
//! ### Automatic Assignment (Discord)
//!
//! When a Discord message is received:
//!
//! 1. The system checks if the channel is mapped to a specific agent
//! 2. If mapped, the agent name and persona are set on the thread
//! 3. If not mapped, the default agent "Al" is used
//!
//! See `server/src/jobs/discord_message_processor.rs` for the implementation.
//!
//! ### Manual Assignment (Code)
//!
//! Use the `ThreadBuilder` to specify an agent:
//!
//! ```rust,ignore
//! // Uses default "Al" agent
//! let thread = ThreadBuilder::new(pool.clone())
//!     .with_goal("Do something")
//!     .build()
//!     .await?;
//!
//! // Override with specific agent
//! let thread = ThreadBuilder::new(pool.clone())
//!     .with_agent("my-agent")
//!     .with_persona("custom-persona")
//!     .with_goal("Do something specific")
//!     .build()
//!     .await?;
//! ```
//!
//! ## Personas
//!
//! Each agent references a persona identifier from the `memory_blocks` table. Personas define:
//! - System prompt personality
//! - Behavior guidelines
//! - Response style
//!
//! Personas are looked up from the `memory_blocks` table where `block_type = 'persona'` and
//! `identifier = agent.persona`.
//!
//! ## Example: "Al" Agent Configuration
//!
//! ```rust,ignore
//! AgentId::Al => AgentConfig {
//!     id: AgentId::Al,
//!     description: "Family standup agent for the family stand-up channel".to_string(),
//!     discord_channel_id: Some(1_040_822_663_658_098_708),
//!     discord_user_id: Some(1_063_930_090_574_061_599),
//!     persona: "default".to_string(),
//!     enabled_tools: Tool::all(), // Al has access to all tools
//! }
//! ```
//!
//! ## Database Schema
//!
//! The `threads` table includes:
//!
//! ```sql
//! agent_name TEXT NOT NULL DEFAULT 'Al'
//! ```
//!
//! This ensures every thread has an associated agent, with "Al" as the default.
//!
//! ## Migration Notes
//!
//! The `agent_name` column was added with a single migration that:
//!
//! 1. Adds the `agent_name` column as nullable
//! 2. Sets all NULL `agent_name` values to "Al" for existing threads
//! 3. Makes the column NOT NULL (ensuring all threads have an agent)
//! 4. Creates an index on `agent_name` for efficient lookups
//!
//! Existing threads continue to work with their assigned agents, and new threads automatically get
//! "Al" unless overridden.
//!
//! See migration: `db/migrations/20251113115315_add_agent_name_to_threads.up.sql`

use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

/// Unique identifier for each agent in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Display, EnumString)]
#[strum(serialize_all = "PascalCase")]
pub enum AgentId {
    Al,
    Demo,
    SecReporting,
}

impl AgentId {
    /// Get the full configuration for this agent
    pub fn config(self) -> AgentConfig {
        match self {
            AgentId::Al => AgentConfig {
                id: AgentId::Al,
                description: "Family standup agent for the family stand-up channel".to_string(),
                discord_channel_id: Some(1_040_822_663_658_098_708),
                discord_user_id: Some(1_063_930_090_574_061_599),
                persona: "default".to_string(),
                enabled_tools: Tool::all(),
            },
            AgentId::Demo => AgentConfig {
                id: AgentId::Demo,
                description: "Demo agent for testing in a different channel".to_string(),
                discord_channel_id: Some(1_438_630_109_312_450_821),
                discord_user_id: Some(1_063_930_090_574_061_599),
                persona: "demo".to_string(),
                enabled_tools: vec![
                    Tool::SendDiscordThreadMessage,
                    Tool::ListenToThread,
                    Tool::ReactToMessage,
                    Tool::ListServerEmojis,
                ],
            },
            AgentId::SecReporting => AgentConfig {
                id: AgentId::SecReporting,
                description: "SEC Reporting Assistant - Automates MD&A narrative generation for quarterly 10-Q filings".to_string(),
                discord_channel_id: None, // TODO: Set this to the actual channel ID when available
                discord_user_id: Some(1_063_930_090_574_061_599),
                persona: "sec-reporting".to_string(),
                enabled_tools: vec![
                    Tool::SendDiscordThreadMessage,
                    Tool::ListenToThread,
                    Tool::ReactToMessage,
                    Tool::ListServerEmojis,
                    Tool::ParseTrialBalance,
                    Tool::CalculateVariances,
                    Tool::GenerateMDAReport,
                    Tool::SaveReport,
                    Tool::CompleteThread,
                ],
            },
        }
    }

    /// Get all available agent IDs
    pub fn all() -> Vec<AgentId> {
        AgentId::iter().collect()
    }
}

/// Default agent ID used when no specific agent is specified
pub const DEFAULT_AGENT_ID: AgentId = AgentId::Al;

/// Available tools that can be used by agents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum Tool {
    // Discord tools for interactive threads
    SendDiscordThreadMessage,
    ListenToThread,
    ReactToMessage,
    ListServerEmojis,

    // Discord tools for regular threads
    SendDiscordMessage,
    CompleteThread,

    // Linear GraphQL tools
    ExecuteLinearQuery,
    SearchLinearQueries,
    SaveLinearQuery,
    ExecuteSavedLinearQuery,
    GetLinearSchema,

    // Cooking tools
    UpsertRecipe,
    GetRecipeByName,
    UpdateInventory,
    GetCookingInventory,
    CreateMealPlan,
    PlanMeal,
    GetAllPlannedMeals,

    // Other tools
    SuggestionsSubmit,

    // SEC Reporting tools
    ParseTrialBalance,
    CalculateVariances,
    GenerateMDAReport,
    SaveReport,
}

impl Tool {
    /// Returns true if this tool should only be available for interactive (Discord) threads
    pub fn is_interactive_only(self) -> bool {
        matches!(
            self,
            Tool::SendDiscordThreadMessage
                | Tool::ListenToThread
                | Tool::ReactToMessage
                | Tool::ListServerEmojis
        )
    }

    /// Returns true if this tool should only be available for autonomous (non-Discord) threads
    pub fn is_autonomous_only(self) -> bool {
        matches!(self, Tool::SendDiscordMessage | Tool::CompleteThread)
    }

    /// Returns true if this tool is available for the given thread type
    pub fn is_available_for_thread_type(
        self,
        thread_type: &db::agentic_threads::ThreadType,
    ) -> bool {
        use db::agentic_threads::ThreadType;

        match thread_type {
            ThreadType::Interactive => !self.is_autonomous_only(),
            ThreadType::Autonomous => !self.is_interactive_only(),
        }
    }
}

impl Tool {
    /// Get the string name of this tool (used for matching with tool calls)
    pub fn name(self) -> &'static str {
        match self {
            Tool::SendDiscordThreadMessage => "send_discord_thread_message",
            Tool::ListenToThread => "listen_to_thread",
            Tool::ReactToMessage => "react_to_message",
            Tool::ListServerEmojis => "list_server_emojis",
            Tool::SendDiscordMessage => "send_discord_message",
            Tool::CompleteThread => "complete_thread",
            Tool::ExecuteLinearQuery => "execute_linear_query",
            Tool::SearchLinearQueries => "search_linear_queries",
            Tool::SaveLinearQuery => "save_linear_query",
            Tool::ExecuteSavedLinearQuery => "execute_saved_linear_query",
            Tool::GetLinearSchema => "get_linear_schema",
            Tool::UpsertRecipe => "upsert_recipe",
            Tool::GetRecipeByName => "get_recipe_by_name",
            Tool::UpdateInventory => "update_inventory",
            Tool::GetCookingInventory => "get_cooking_inventory",
            Tool::CreateMealPlan => "create_meal_plan",
            Tool::PlanMeal => "plan_meal",
            Tool::GetAllPlannedMeals => "get_all_planned_meals",
            Tool::SuggestionsSubmit => "tool_suggestions_submit",
            Tool::ParseTrialBalance => "parse_trial_balance",
            Tool::CalculateVariances => "calculate_variances",
            Tool::GenerateMDAReport => "generate_mda_report",
            Tool::SaveReport => "save_report",
        }
    }

    /// Get all available tools
    /// This is automatically generated from the enum variants using strum,
    /// so it's impossible to forget to add a new tool to this list.
    pub fn all() -> Vec<Tool> {
        Tool::iter().collect()
    }

    /// Create an instance of the actual tool implementation for this tool enum variant.
    /// Returns a boxed `GenericTool` that can be added to a `ToolBag`.
    ///
    /// This centralizes tool instantiation so we don't have to manually check and add
    /// each tool in the thread processor.
    pub fn create_instance(self) -> Box<dyn crate::al::tools::GenericTool> {
        use crate::al::tools::{
            cooking_simple::{
                AddRecipeToMealPlan, CheckInventory, CreateMealPlan, GetRecipe, ListMealPlans,
                UpdateInventory, UpsertRecipe,
            },
            discord::{
                ListServerEmojis, ListenToThread, ReactToMessage, SendDiscordMessage,
                SendDiscordThreadMessage,
            },
            linear_graphql::{
                ExecuteLinearQuery, ExecuteSavedLinearQuery, GetLinearSchema, SaveLinearQuery,
                SearchLinearQueries,
            },
            sec_reporting::{
                CalculateVariances, GenerateMDAReport, ParseTrialBalance, SaveReport,
            },
            threads::CompleteThread,
            Tool as ToolTrait,
        };

        match self {
            Tool::SendDiscordThreadMessage => SendDiscordThreadMessage::new().to_generic(),
            Tool::ListenToThread => ListenToThread::new().to_generic(),
            Tool::ReactToMessage => ReactToMessage::new().to_generic(),
            Tool::ListServerEmojis => ListServerEmojis::new().to_generic(),
            Tool::SendDiscordMessage => SendDiscordMessage.to_generic(),
            Tool::CompleteThread => CompleteThread::new().to_generic(),
            Tool::ExecuteLinearQuery => ExecuteLinearQuery.to_generic(),
            Tool::SearchLinearQueries => SearchLinearQueries.to_generic(),
            Tool::SaveLinearQuery => SaveLinearQuery.to_generic(),
            Tool::ExecuteSavedLinearQuery => ExecuteSavedLinearQuery.to_generic(),
            Tool::GetLinearSchema => GetLinearSchema.to_generic(),
            Tool::UpsertRecipe => UpsertRecipe.to_generic(),
            Tool::GetRecipeByName => GetRecipe.to_generic(),
            Tool::UpdateInventory => UpdateInventory.to_generic(),
            Tool::GetCookingInventory => CheckInventory.to_generic(),
            Tool::CreateMealPlan => CreateMealPlan.to_generic(),
            Tool::PlanMeal => AddRecipeToMealPlan.to_generic(),
            Tool::GetAllPlannedMeals => ListMealPlans.to_generic(),
            Tool::SuggestionsSubmit => {
                crate::al::tools::tool_suggestions::ToolSuggestionsSubmit::new().to_generic()
            }
            Tool::ParseTrialBalance => ParseTrialBalance.to_generic(),
            Tool::CalculateVariances => CalculateVariances.to_generic(),
            Tool::GenerateMDAReport => GenerateMDAReport.to_generic(),
            Tool::SaveReport => SaveReport.to_generic(),
        }
    }
}

/// Configuration for a single AI agent
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Unique identifier for the agent
    pub id: AgentId,

    /// Human-readable description
    pub description: String,

    /// Discord channel ID where this agent operates (for autonomous messages like standups)
    pub discord_channel_id: Option<u64>,

    /// Discord user ID for the bot when posting
    pub discord_user_id: Option<u64>,

    /// Persona identifier to use from `memory_blocks` table
    pub persona: String,

    /// List of enabled tools for this agent
    pub enabled_tools: Vec<Tool>,
}

impl AgentConfig {
    /// Check if a specific tool is enabled for this agent
    pub fn has_tool(&self, tool: Tool) -> bool {
        self.enabled_tools.contains(&tool)
    }

    /// Create a Discord thread and return a pre-configured `ThreadBuilder`
    ///
    /// This creates a Discord thread in the agent's configured channel and returns
    /// a `ThreadBuilder` that's already set up with the agent ID and Discord metadata.
    /// The caller should add a goal and any other configuration before building.
    ///
    /// # Example
    /// ```rust,ignore
    /// let config = AgentId::Al.config();
    /// let thread = config
    ///     .create_thread(&discord_client, &db_pool, "My Thread".to_string())
    ///     .await?
    ///     .with_goal("Do something")
    ///     .build()
    ///     .await?;
    /// ```
    pub async fn create_thread(
        &self,
        discord_client: &crate::discord::DiscordClient,
        db_pool: &sqlx::PgPool,
        thread_name: String,
    ) -> cja::Result<crate::agentic_threads::ThreadBuilder> {
        use serenity::all::Channel;

        let channel_id = self.discord_channel_id.ok_or_else(|| {
            cja::color_eyre::eyre::eyre!(
                "Agent '{:?}' does not have a discord_channel_id configured",
                self.id
            )
        })?;

        let channel_id = serenity::all::ChannelId::new(channel_id);
        let channel = channel_id.to_channel(discord_client).await?;
        let Channel::Guild(guild_channel) = channel else {
            cja::color_eyre::eyre::bail!("Channel is not a guild channel");
        };

        let builder = serenity::all::CreateThread::new(thread_name.clone())
            .auto_archive_duration(serenity::all::AutoArchiveDuration::OneDay);
        let new_discord_thread = guild_channel.create_thread(discord_client, builder).await?;

        let discord_metadata = crate::agentic_threads::builder::DiscordMetadata {
            discord_thread_id: new_discord_thread.id.to_string(),
            channel_id: guild_channel.id.to_string(),
            guild_id: guild_channel.guild_id.to_string(),
            created_by: self.id.to_string(),
            thread_name,
        };

        Ok(crate::agentic_threads::ThreadBuilder::new(db_pool.clone())
            .with_agent(self.id)
            .interactive_discord(discord_metadata))
    }
}

/// Get an agent ID by Discord channel ID
/// Returns None if the channel doesn't have an associated agent
pub fn get_agent_by_channel(channel_id: u64) -> Option<AgentId> {
    AgentId::all()
        .into_iter()
        .find(|agent_id| agent_id.config().discord_channel_id == Some(channel_id))
}
