use db::models::{LinearQueryUsage, LinearSavedQuery, LinearSavedQueryWithStats};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::types::Uuid;
use std::time::Instant;

use crate::{
    al::tools::{ThreadContext, Tool},
    AppState,
};

// Tool for executing GraphQL queries against Linear API
pub struct ExecuteLinearQuery;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecuteLinearQueryInput {
    /// The GraphQL query to execute
    query: String,
    /// Variables for the GraphQL query
    #[serde(default)]
    variables: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteLinearQueryOutput {
    /// The response data from the query
    data: Option<JsonValue>,
    /// Any errors from the query
    errors: Option<Vec<GraphQLError>>,
    /// Execution time in milliseconds
    execution_time_ms: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLError {
    message: String,
    #[serde(default)]
    extensions: JsonValue,
}

#[async_trait::async_trait]
impl Tool for ExecuteLinearQuery {
    const NAME: &'static str = "execute_linear_query";
    const DESCRIPTION: &'static str = "Execute a GraphQL query against the Linear API";

    type ToolInput = ExecuteLinearQueryInput;
    type ToolOutput = ExecuteLinearQueryOutput;

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        _context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        let start = Instant::now();

        // Get Linear API key from environment or thread metadata
        let api_key = get_linear_api_key(&app_state).await?;

        // Execute the GraphQL query
        let client = reqwest::Client::builder()
            .user_agent("github.com/coreyja/coreyja.com")
            .default_headers(
                std::iter::once((
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Bearer {api_key}"))?,
                ))
                .collect(),
            )
            .build()?;

        let body = serde_json::json!({
            "query": input.query,
            "variables": input.variables,
        });

        let response = client
            .post("https://api.linear.app/graphql")
            .json(&body)
            .send()
            .await?;

        let execution_time_ms = start.elapsed().as_millis();
        let _response_bytes = response.content_length().unwrap_or(0);
        let _status = response.status();
        let response_json: JsonValue = response.json().await?;

        // Extract data and errors from response
        let data = response_json.get("data").cloned();
        let errors = response_json
            .get("errors")
            .and_then(|e| serde_json::from_value::<Vec<GraphQLError>>(e.clone()).ok());

        Ok(ExecuteLinearQueryOutput {
            data,
            errors,
            execution_time_ms: execution_time_ms.to_string(),
        })
    }
}

// Tool for searching saved queries
pub struct SearchLinearQueries;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchLinearQueriesInput {
    /// Search term to match against query names and descriptions
    #[serde(default)]
    search_term: Option<String>,
    /// Filter by tags
    #[serde(default)]
    tags: Option<Vec<String>>,
    /// Maximum number of results to return (default: 10, max: 50)
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchLinearQueriesOutput {
    queries: Vec<SavedQueryInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedQueryInfo {
    id: Uuid,
    name: String,
    description: Option<String>,
    query: String,
    variables_schema: Option<JsonValue>,
    tags: Option<Vec<String>>,
    last_used: Option<chrono::DateTime<chrono::Utc>>,
    use_count: i64,
}

#[async_trait::async_trait]
impl Tool for SearchLinearQueries {
    const NAME: &'static str = "search_linear_queries";
    const DESCRIPTION: &'static str = "Search for saved Linear GraphQL queries";

    type ToolInput = SearchLinearQueriesInput;
    type ToolOutput = SearchLinearQueriesOutput;

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        _context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        let mut conn = app_state.db.acquire().await?;

        let limit = input.limit.clamp(1, 50);

        let queries = LinearSavedQueryWithStats::find_with_stats(
            &mut conn,
            input.search_term.as_deref(),
            input.tags.as_deref(),
            limit,
        )
        .await?;

        let query_infos = queries
            .into_iter()
            .map(|q| SavedQueryInfo {
                id: q.query.id,
                name: q.query.name,
                description: q.query.description,
                query: q.query.query,
                variables_schema: q.query.variables_schema,
                tags: q.query.tags,
                last_used: q.last_used,
                use_count: q.use_count,
            })
            .collect();

        Ok(SearchLinearQueriesOutput {
            queries: query_infos,
        })
    }
}

// Tool for saving a new query
pub struct SaveLinearQuery;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SaveLinearQueryInput {
    /// Unique name for the query
    name: String,
    /// Description of what the query does
    #[serde(default)]
    description: Option<String>,
    /// The GraphQL query
    query: String,
    /// JSON Schema for the variables this query accepts
    #[serde(default)]
    variables_schema: Option<JsonValue>,
    /// Tags to categorize the query
    #[serde(default)]
    tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveLinearQueryOutput {
    id: Uuid,
    name: String,
    message: String,
}

#[async_trait::async_trait]
impl Tool for SaveLinearQuery {
    const NAME: &'static str = "save_linear_query";
    const DESCRIPTION: &'static str = "Save a new Linear GraphQL query for later use";

    type ToolInput = SaveLinearQueryInput;
    type ToolOutput = SaveLinearQueryOutput;

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        _context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        let mut conn = app_state.db.acquire().await?;

        // Check if a query with this name already exists
        if let Some(existing) = LinearSavedQuery::find_by_name(&mut conn, &input.name).await? {
            return Err(cja::color_eyre::eyre::eyre!(
                "A query with the name '{}' already exists (id: {})",
                input.name,
                existing.id
            ));
        }

        // Save the new query
        let saved_query = LinearSavedQuery::create(
            &mut conn,
            input.name,
            input.description,
            input.query,
            input.variables_schema,
            input.tags,
            Some("al".to_string()), // Mark as created by Al
        )
        .await?;

        Ok(SaveLinearQueryOutput {
            id: saved_query.id,
            name: saved_query.name.clone(),
            message: format!("Successfully saved query '{}'", saved_query.name),
        })
    }
}

// Tool for executing a saved query by name or ID
pub struct ExecuteSavedLinearQuery;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecuteSavedLinearQueryInput {
    /// The name or ID of the saved query to execute
    query_identifier: String,
    /// Variables for the GraphQL query
    #[serde(default)]
    variables: JsonValue,
}

#[async_trait::async_trait]
impl Tool for ExecuteSavedLinearQuery {
    const NAME: &'static str = "execute_saved_linear_query";
    const DESCRIPTION: &'static str = "Execute a saved Linear GraphQL query by name or ID";

    type ToolInput = ExecuteSavedLinearQueryInput;
    type ToolOutput = ExecuteLinearQueryOutput;

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        let mut conn = app_state.db.acquire().await?;

        // Try to find the query by ID first, then by name
        let saved_query = if let Ok(id) = Uuid::parse_str(&input.query_identifier) {
            LinearSavedQuery::find_by_id(&mut conn, id).await?
        } else {
            LinearSavedQuery::find_by_name(&mut conn, &input.query_identifier).await?
        };

        let saved_query = saved_query.ok_or_else(|| {
            cja::color_eyre::eyre::eyre!(
                "No saved query found with identifier '{}'",
                input.query_identifier
            )
        })?;

        // Execute the query
        let execute_tool = ExecuteLinearQuery;

        let result = execute_tool
            .run(
                ExecuteLinearQueryInput {
                    query: saved_query.query.clone(),
                    variables: input.variables.clone(),
                },
                app_state.clone(),
                context,
            )
            .await;

        // Track usage
        let success = result.is_ok();
        let error_message = result.as_ref().err().map(std::string::ToString::to_string);

        LinearQueryUsage::create(
            &mut conn,
            saved_query.id,
            Some(input.variables),
            success,
            error_message,
        )
        .await?;

        result
    }
}

// Tool for fetching the Linear GraphQL schema
pub struct GetLinearSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetLinearSchemaInput {
    /// Type to get schema for (leave empty for full schema)
    #[serde(default)]
    type_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLinearSchemaOutput {
    schema: String,
}

#[async_trait::async_trait]
impl Tool for GetLinearSchema {
    const NAME: &'static str = "get_linear_schema";
    const DESCRIPTION: &'static str =
        "Get the Linear GraphQL schema or information about specific types";

    type ToolInput = GetLinearSchemaInput;
    type ToolOutput = GetLinearSchemaOutput;

    async fn run(
        &self,
        input: Self::ToolInput,
        app_state: AppState,
        _context: ThreadContext,
    ) -> cja::Result<Self::ToolOutput> {
        let api_key = get_linear_api_key(&app_state).await?;

        // Build introspection query
        let query = if let Some(type_name) = input.type_name {
            // Get schema for specific type
            format!(
                r#"
                query IntrospectionQuery {{
                    __type(name: "{type_name}") {{
                        name
                        kind
                        description
                        fields {{
                            name
                            description
                            type {{
                                name
                                kind
                                ofType {{
                                    name
                                    kind
                                }}
                            }}
                            args {{
                                name
                                description
                                type {{
                                    name
                                    kind
                                    ofType {{
                                        name
                                        kind
                                    }}
                                }}
                            }}
                        }}
                    }}
                }}
                "#
            )
        } else {
            // Get basic schema information
            r"
            query IntrospectionQuery {
                __schema {
                    queryType {
                        name
                        fields {
                            name
                            description
                        }
                    }
                    mutationType {
                        name
                        fields {
                            name
                            description
                        }
                    }
                    types {
                        name
                        kind
                        description
                    }
                }
            }
            "
            .to_string()
        };

        let client = reqwest::Client::builder()
            .user_agent("github.com/coreyja/coreyja.com")
            .default_headers(
                std::iter::once((
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Bearer {api_key}"))?,
                ))
                .collect(),
            )
            .build()?;

        let body = serde_json::json!({
            "query": query,
            "variables": {}
        });

        let response = client
            .post("https://api.linear.app/graphql")
            .json(&body)
            .send()
            .await?;

        let response_json: JsonValue = response.json().await?;

        // Format the schema information
        let schema = serde_json::to_string_pretty(&response_json["data"])?;

        Ok(GetLinearSchemaOutput { schema })
    }
}

pub async fn get_linear_api_key(app_state: &AppState) -> cja::Result<String> {
    // TODO: We would ideally like to not just grab the first installation. But I'll only have 1 for now so it'll be fine

    let installation = sqlx::query!(
        r#"
            SELECT encrypted_access_token
            FROM linear_installations
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
    )
    .fetch_optional(&app_state.db)
    .await?;

    let Some(installation) = installation else {
        color_eyre::eyre::bail!("No Linear API key found");
    };

    use crate::encrypt::decrypt;
    let access_token = decrypt(
        &installation.encrypted_access_token,
        &app_state.encrypt_config,
    )?;
    Ok(access_token)
}
