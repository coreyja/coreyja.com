use cja::Result;
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use reqwest::Client;

use super::agent::AgentActivityContent;

type DateTime = chrono::DateTime<chrono::Utc>;
type JSONObject = serde_json::Value;

// The paths are relative to the directory where your `Cargo.toml` is located.
// Both json and the GraphQL schema language are supported as sources for the schema
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/linear/schema.graphql",
    query_path = "src/linear/get_workspace.graphql",
    response_derives = "Debug"
)]
pub struct GetWorkspace;

pub async fn get_workspace(access_token: &str) -> Result<get_workspace::ResponseData> {
    let client = Client::builder()
        .user_agent("github.com/coreyja/coreyja.com")
        .default_headers(
            std::iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {access_token}"))?,
            ))
            .collect(),
        )
        .build()?;

    let response = post_graphql::<GetWorkspace, _>(
        &client,
        "https://api.linear.app/graphql",
        get_workspace::Variables {},
    )
    .await?;

    response
        .data
        .ok_or_else(|| cja::color_eyre::eyre::eyre!("No data returned from Linear API"))
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/linear/schema.graphql",
    query_path = "src/linear/me.graphql",
    response_derives = "Debug"
)]
pub struct Me;

pub async fn get_me(access_token: &str) -> Result<me::ResponseData> {
    let client = Client::builder()
        .user_agent("github.com/coreyja/coreyja.com")
        .default_headers(
            std::iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {access_token}"))?,
            ))
            .collect(),
        )
        .build()?;

    let response =
        post_graphql::<Me, _>(&client, "https://api.linear.app/graphql", me::Variables {}).await?;

    response
        .data
        .ok_or_else(|| cja::color_eyre::eyre::eyre!("No data returned from Linear API"))
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/linear/schema.graphql",
    query_path = "src/linear/mutations/agent_activity_create.graphql",
    response_derives = "Debug",
    variables_derives = "Debug"
)]
pub struct AgentActivityCreate;

pub async fn create_agent_activity(
    api_key: &str,
    agent_session_id: impl Into<String>,
    content: AgentActivityContent,
) -> Result<()> {
    let client = Client::builder()
        .user_agent("github.com/coreyja/coreyja.com")
        .default_headers(
            std::iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {api_key}"))?,
            ))
            .collect(),
        )
        .build()?;

    // Serialize the content to JSON for the GraphQL input
    let content_json = serde_json::to_value(&content)?;

    let agent_session_id = agent_session_id.into();

    let variables = agent_activity_create::Variables {
        input: agent_activity_create::AgentActivityCreateInput {
            id: Some(uuid::Uuid::new_v4().to_string()),
            agent_session_id,
            content: content_json,
        },
    };

    let response = post_graphql::<AgentActivityCreate, _>(
        &client,
        "https://api.linear.app/graphql",
        variables,
    )
    .await?;

    if let Some(data) = response.data {
        if !data.agent_activity_create.success {
            return Err(cja::color_eyre::eyre::eyre!(
                "Failed to create agent activity"
            ));
        }
    } else if let Some(errors) = response.errors {
        return Err(cja::color_eyre::eyre::eyre!("GraphQL errors: {:?}", errors));
    } else {
        return Err(cja::color_eyre::eyre::eyre!(
            "No data returned from Linear API"
        ));
    }

    Ok(())
}
