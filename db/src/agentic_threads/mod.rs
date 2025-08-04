use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use sqlx::{types::Uuid, PgPool, Type};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "snake_case")]
pub enum StitchType {
    #[serde(rename = "initial_prompt")]
    InitialPrompt,
    #[serde(rename = "llm_call")]
    LlmCall,
    #[serde(rename = "tool_call")]
    ToolCall,
    #[serde(rename = "thread_result")]
    ThreadResult,
    #[serde(rename = "discord_message")]
    DiscordMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "snake_case")]
pub enum ThreadType {
    #[serde(rename = "autonomous")]
    Autonomous,
    #[serde(rename = "interactive")]
    Interactive,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "snake_case")]
pub enum ThreadStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "waiting")]
    Waiting,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "aborted")]
    Aborted,
}

impl fmt::Display for StitchType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StitchType::InitialPrompt => write!(f, "initial_prompt"),
            StitchType::LlmCall => write!(f, "llm_call"),
            StitchType::ToolCall => write!(f, "tool_call"),
            StitchType::ThreadResult => write!(f, "thread_result"),
            StitchType::DiscordMessage => write!(f, "discord_message"),
        }
    }
}

impl std::str::FromStr for StitchType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "initial_prompt" => Ok(StitchType::InitialPrompt),
            "llm_call" => Ok(StitchType::LlmCall),
            "tool_call" => Ok(StitchType::ToolCall),
            "thread_result" => Ok(StitchType::ThreadResult),
            "discord_message" => Ok(StitchType::DiscordMessage),
            _ => Err(format!("Unknown stitch type: {s}")),
        }
    }
}

impl From<String> for StitchType {
    fn from(s: String) -> Self {
        s.parse().expect("Invalid stitch type")
    }
}

impl fmt::Display for ThreadType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThreadType::Autonomous => write!(f, "autonomous"),
            ThreadType::Interactive => write!(f, "interactive"),
        }
    }
}

impl std::str::FromStr for ThreadType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "autonomous" => Ok(ThreadType::Autonomous),
            "interactive" => Ok(ThreadType::Interactive),
            _ => Err(format!("Unknown thread type: {s}")),
        }
    }
}

impl From<String> for ThreadType {
    fn from(s: String) -> Self {
        s.parse().expect("Invalid thread type")
    }
}

impl fmt::Display for ThreadStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThreadStatus::Pending => write!(f, "pending"),
            ThreadStatus::Running => write!(f, "running"),
            ThreadStatus::Waiting => write!(f, "waiting"),
            ThreadStatus::Completed => write!(f, "completed"),
            ThreadStatus::Failed => write!(f, "failed"),
            ThreadStatus::Aborted => write!(f, "aborted"),
        }
    }
}

impl std::str::FromStr for ThreadStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(ThreadStatus::Pending),
            "running" => Ok(ThreadStatus::Running),
            "waiting" => Ok(ThreadStatus::Waiting),
            "completed" => Ok(ThreadStatus::Completed),
            "failed" => Ok(ThreadStatus::Failed),
            "aborted" => Ok(ThreadStatus::Aborted),
            _ => Err(format!("Unknown thread status: {s}")),
        }
    }
}

impl From<String> for ThreadStatus {
    fn from(s: String) -> Self {
        s.parse().expect("Invalid thread status")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Thread {
    pub thread_id: Uuid,
    pub branching_stitch_id: Option<Uuid>,
    pub goal: String,
    pub tasks: JsonValue,
    pub status: ThreadStatus,
    pub result: Option<JsonValue>,
    pub pending_child_results: JsonValue,
    pub thread_type: ThreadType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Stitch {
    pub stitch_id: Uuid,
    pub thread_id: Uuid,
    pub previous_stitch_id: Option<Uuid>,
    pub stitch_type: StitchType,
    pub llm_request: Option<JsonValue>,
    pub llm_response: Option<JsonValue>,
    pub tool_name: Option<String>,
    pub tool_input: Option<JsonValue>,
    pub tool_output: Option<JsonValue>,
    pub child_thread_id: Option<Uuid>,
    pub thread_result_summary: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Thread {
    pub async fn create(
        pool: &PgPool,
        goal: String,
        branching_stitch_id: Option<Uuid>,
        thread_type: Option<ThreadType>,
    ) -> color_eyre::Result<Self> {
        let thread_type_str =
            thread_type.map_or_else(|| "autonomous".to_string(), |t| t.to_string());

        let thread = sqlx::query_as!(
            Thread,
            r#"
            INSERT INTO threads (goal, branching_stitch_id, thread_type)
            VALUES ($1, $2, $3)
            RETURNING 
                *
            "#,
            goal,
            branching_stitch_id,
            thread_type_str
        )
        .fetch_one(pool)
        .await?;

        Ok(thread)
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> color_eyre::Result<Option<Self>> {
        let thread = sqlx::query_as!(
            Thread,
            r#"
            SELECT 
                *
            FROM threads
            WHERE thread_id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(thread)
    }

    pub async fn list_all(pool: &PgPool) -> color_eyre::Result<Vec<Self>> {
        let threads = sqlx::query_as!(
            Thread,
            r#"
            SELECT 
                *
            FROM threads
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(threads)
    }

    pub async fn update_status(
        pool: &PgPool,
        id: Uuid,
        status: &str,
    ) -> color_eyre::Result<Option<Self>> {
        let thread = sqlx::query_as!(
            Thread,
            r#"
            UPDATE threads
            SET status = $1, updated_at = NOW()
            WHERE thread_id = $2
            RETURNING 
                *
            "#,
            status,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(thread)
    }

    pub async fn update_tasks(
        pool: &PgPool,
        id: Uuid,
        tasks: JsonValue,
    ) -> color_eyre::Result<Option<Self>> {
        let thread = sqlx::query_as!(
            Thread,
            r#"
            UPDATE threads
            SET tasks = $1, updated_at = NOW()
            WHERE thread_id = $2
            RETURNING 
                *
            "#,
            tasks,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(thread)
    }

    pub async fn complete(
        pool: &PgPool,
        id: Uuid,
        result: JsonValue,
    ) -> color_eyre::Result<Option<Self>> {
        let thread = sqlx::query_as!(
            Thread,
            r#"
            UPDATE threads
            SET status = 'completed', result = $1, updated_at = NOW()
            WHERE thread_id = $2
            RETURNING 
                *
            "#,
            result,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(thread)
    }

    pub async fn fail(
        pool: &PgPool,
        id: Uuid,
        result: JsonValue,
    ) -> color_eyre::Result<Option<Self>> {
        let thread = sqlx::query_as!(
            Thread,
            r#"
            UPDATE threads
            SET status = 'failed', result = $1, updated_at = NOW()
            WHERE thread_id = $2
            RETURNING 
                *
            "#,
            result,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(thread)
    }

    pub async fn get_stitches(&self, pool: &PgPool) -> color_eyre::Result<Vec<Stitch>> {
        Stitch::get_by_thread_ordered(pool, self.thread_id).await
    }

    pub async fn get_parent_thread(&self, pool: &PgPool) -> color_eyre::Result<Option<Self>> {
        if let Some(branching_stitch_id) = self.branching_stitch_id {
            let parent_stitch = sqlx::query!(
                r#"
                SELECT thread_id FROM stitches
                WHERE stitch_id = $1
                "#,
                branching_stitch_id
            )
            .fetch_optional(pool)
            .await?;

            if let Some(parent) = parent_stitch {
                return Thread::get_by_id(pool, parent.thread_id).await;
            }
        }
        Ok(None)
    }

    pub async fn get_parent_chain(&self, pool: &PgPool) -> color_eyre::Result<Vec<Self>> {
        let mut chain = Vec::new();
        let mut current_thread = self.clone();

        while let Some(parent) = current_thread.get_parent_thread(pool).await? {
            chain.push(parent.clone());
            current_thread = parent;
        }

        chain.reverse(); // Return root-to-child order
        Ok(chain)
    }

    pub async fn list_recent_top_level(pool: &PgPool, limit: i64) -> color_eyre::Result<Vec<Self>> {
        let threads = sqlx::query_as!(
            Thread,
            r#"
            SELECT 
                *
            FROM threads
            WHERE branching_stitch_id IS NULL
            ORDER BY created_at DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(pool)
        .await?;

        Ok(threads)
    }

    pub async fn get_children(&self, pool: &PgPool) -> color_eyre::Result<Vec<Self>> {
        let children = sqlx::query_as!(
            Thread,
            r#"
            SELECT 
                t.*
            FROM threads t
            JOIN stitches s ON t.branching_stitch_id = s.stitch_id
            WHERE s.thread_id = $1
            ORDER BY t.created_at ASC
            "#,
            self.thread_id
        )
        .fetch_all(pool)
        .await?;

        Ok(children)
    }

    pub async fn count_children(&self, pool: &PgPool) -> color_eyre::Result<i64> {
        let count = sqlx::query!(
            r#"
            SELECT COUNT(*) as count FROM threads t
            JOIN stitches s ON t.branching_stitch_id = s.stitch_id
            WHERE s.thread_id = $1
            "#,
            self.thread_id
        )
        .fetch_one(pool)
        .await?;

        Ok(count.count.unwrap_or(0))
    }

    pub async fn count_stitches(&self, pool: &PgPool) -> color_eyre::Result<i64> {
        let count = sqlx::query!(
            r#"
            SELECT COUNT(*) as count FROM stitches
            WHERE thread_id = $1
            "#,
            self.thread_id
        )
        .fetch_one(pool)
        .await?;

        Ok(count.count.unwrap_or(0))
    }

    pub async fn abort(
        pool: &PgPool,
        id: Uuid,
        result: JsonValue,
    ) -> color_eyre::Result<Option<Self>> {
        let thread = sqlx::query_as!(
            Thread,
            r#"
            UPDATE threads
            SET status = 'aborted', result = $1, updated_at = NOW()
            WHERE thread_id = $2
            RETURNING 
                *
            "#,
            result,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(thread)
    }
}

impl Stitch {
    pub async fn create_llm_call(
        pool: &PgPool,
        thread_id: Uuid,
        previous_stitch_id: Option<Uuid>,
        llm_request: JsonValue,
        llm_response: JsonValue,
    ) -> color_eyre::Result<Self> {
        let stitch = sqlx::query_as!(
            Stitch,
            r#"
            INSERT INTO stitches (thread_id, previous_stitch_id, stitch_type, llm_request, llm_response)
            VALUES ($1, $2, 'llm_call', $3, $4)
            RETURNING *
            "#,
            thread_id,
            previous_stitch_id,
            llm_request,
            llm_response
        )
        .fetch_one(pool)
        .await?;

        Ok(stitch)
    }

    pub async fn create_tool_call(
        pool: &PgPool,
        thread_id: Uuid,
        previous_stitch_id: Option<Uuid>,
        tool_name: String,
        tool_input: JsonValue,
        tool_output: JsonValue,
    ) -> color_eyre::Result<Self> {
        let stitch = sqlx::query_as!(
            Stitch,
            r#"
            INSERT INTO stitches (thread_id, previous_stitch_id, stitch_type, tool_name, tool_input, tool_output)
            VALUES ($1, $2, 'tool_call', $3, $4, $5)
            RETURNING *
            "#,
            thread_id,
            previous_stitch_id,
            tool_name,
            tool_input,
            tool_output
        )
        .fetch_one(pool)
        .await?;

        Ok(stitch)
    }

    pub async fn create_thread_result(
        pool: &PgPool,
        thread_id: Uuid,
        previous_stitch_id: Option<Uuid>,
        child_thread_id: Uuid,
        thread_result_summary: String,
    ) -> color_eyre::Result<Self> {
        let stitch = sqlx::query_as!(
            Stitch,
            r#"
            INSERT INTO stitches (thread_id, previous_stitch_id, stitch_type, child_thread_id, thread_result_summary)
            VALUES ($1, $2, 'thread_result', $3, $4)
            RETURNING *
            "#,
            thread_id,
            previous_stitch_id,
            child_thread_id,
            thread_result_summary
        )
        .fetch_one(pool)
        .await?;

        Ok(stitch)
    }

    pub async fn create_initial_user_message(
        pool: &PgPool,
        thread_id: Uuid,
        prompt: String,
    ) -> color_eyre::Result<Self> {
        // Create the request JSON with the user's initial message
        let request = json!({
            "messages": [{
                "role": "user",
                "content": [{
                    "type": "text",
                    "text": prompt
                }]
            }]
        });

        // Create an empty response for now
        let response = json!({
            "content": []
        });

        // Create the initial prompt stitch with no previous stitch
        let stitch = sqlx::query_as!(
            Stitch,
            r#"
            INSERT INTO stitches (thread_id, previous_stitch_id, stitch_type, llm_request, llm_response)
            VALUES ($1, $2, 'initial_prompt', $3, $4)
            RETURNING *
            "#,
            thread_id,
            None::<Uuid>,
            request,
            response
        )
        .fetch_one(pool)
        .await?;

        Ok(stitch)
    }

    pub async fn create_discord_message(
        pool: &PgPool,
        thread_id: Uuid,
        previous_stitch_id: Option<Uuid>,
        message_data: JsonValue,
    ) -> color_eyre::Result<Self> {
        let stitch = sqlx::query_as!(
            Stitch,
            r#"
            INSERT INTO stitches (thread_id, previous_stitch_id, stitch_type, llm_request)
            VALUES ($1, $2, 'discord_message', $3)
            RETURNING *
            "#,
            thread_id,
            previous_stitch_id,
            message_data
        )
        .fetch_one(pool)
        .await?;

        Ok(stitch)
    }

    pub async fn create_system_prompt(
        pool: &PgPool,
        thread_id: Uuid,
        system_prompt: String,
    ) -> color_eyre::Result<Self> {
        let request = json!({
            "messages": [{
                "role": "system",
                "content": [{
                    "type": "text",
                    "text": system_prompt
                }]
            }]
        });

        let stitch = sqlx::query_as!(
            Stitch,
            r#"
            INSERT INTO stitches (thread_id, previous_stitch_id, stitch_type, llm_request)
            VALUES ($1, NULL, 'initial_prompt', $2)
            RETURNING *
            "#,
            thread_id,
            request
        )
        .fetch_one(pool)
        .await?;

        Ok(stitch)
    }

    pub async fn get_last_stitch(
        pool: &PgPool,
        thread_id: Uuid,
    ) -> color_eyre::Result<Option<Self>> {
        let stitch = sqlx::query_as!(
            Stitch,
            r#"
            SELECT * FROM stitches 
            WHERE thread_id = $1 
            AND stitch_id NOT IN (
                SELECT previous_stitch_id FROM stitches 
                WHERE thread_id = $1 AND previous_stitch_id IS NOT NULL
            )
            "#,
            thread_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(stitch)
    }

    pub async fn get_by_thread_ordered(
        pool: &PgPool,
        thread_id: Uuid,
    ) -> color_eyre::Result<Vec<Self>> {
        let stitches = sqlx::query_as!(
            Stitch,
            r#"
            WITH RECURSIVE thread_history AS (
                SELECT * FROM stitches WHERE thread_id = $1 AND previous_stitch_id IS NULL
                UNION ALL
                SELECT s.* FROM stitches s
                JOIN thread_history th ON s.previous_stitch_id = th.stitch_id
            )
            SELECT 
                stitch_id as "stitch_id!",
                thread_id as "thread_id!",
                previous_stitch_id,
                stitch_type as "stitch_type!: StitchType",
                llm_request,
                llm_response,
                tool_name,
                tool_input,
                tool_output,
                child_thread_id,
                thread_result_summary,
                created_at as "created_at!"
            FROM thread_history 
            ORDER BY created_at
            "#,
            thread_id
        )
        .fetch_all(pool)
        .await?;

        Ok(stitches)
    }
}
