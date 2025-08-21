use cja::Result;
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use reqwest::Client;

use super::agent::AgentActivityContent;

type DateTime = chrono::DateTime<chrono::Utc>;
type JSONObject = serde_json::Value;
type TimelessDate = String; // Linear uses YYYY-MM-DD format for dates

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

// Standup query parts
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/linear/schema.graphql",
    query_path = "src/linear/standup/viewer_and_team.graphql",
    response_derives = "Debug, Serialize"
)]
pub struct StandupViewerAndTeam;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/linear/schema.graphql",
    query_path = "src/linear/standup/active_cycle.graphql",
    response_derives = "Debug, Serialize"
)]
pub struct StandupActiveCycle;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/linear/schema.graphql",
    query_path = "src/linear/standup/previous_cycle.graphql",
    response_derives = "Debug, Serialize"
)]
pub struct StandupPreviousCycle;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/linear/schema.graphql",
    query_path = "src/linear/standup/in_progress_issues.graphql",
    response_derives = "Debug, Serialize"
)]
pub struct StandupInProgressIssues;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/linear/schema.graphql",
    query_path = "src/linear/standup/recently_updated_issues.graphql",
    response_derives = "Debug, Serialize"
)]
pub struct StandupRecentlyUpdatedIssues;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/linear/schema.graphql",
    query_path = "src/linear/standup/backlog_issues.graphql",
    response_derives = "Debug, Serialize"
)]
pub struct StandupBacklogIssues;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/linear/schema.graphql",
    query_path = "src/linear/standup/issues_with_blockers.graphql",
    response_derives = "Debug, Serialize"
)]
pub struct StandupIssuesWithBlockers;

// Combined standup data structure
#[derive(Debug, serde::Serialize)]
pub struct StandupData {
    pub viewer: Option<standup_viewer_and_team::StandupViewerAndTeamViewer>,
    pub team: Option<standup_viewer_and_team::StandupViewerAndTeamTeam>,
    pub active_cycle: Option<standup_active_cycle::StandupActiveCycleTeamActiveCycle>,
    pub previous_cycle: Vec<standup_previous_cycle::StandupPreviousCycleTeamPreviousCycleNodes>,
    pub in_progress_issues: Option<Vec<standup_in_progress_issues::IssueWithRelations>>,
    pub recently_updated_issues: Option<Vec<standup_recently_updated_issues::IssueCompact>>,
    pub backlog_issues: Option<Vec<standup_backlog_issues::IssueSimple>>,
    pub issues_with_blockers: Option<Vec<standup_issues_with_blockers::IssueWithBlockers>>,
}

async fn create_linear_client(access_token: &str) -> Result<Client> {
    Client::builder()
        .user_agent("github.com/coreyja/coreyja.com")
        .default_headers(
            std::iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {access_token}"))?,
            ))
            .collect(),
        )
        .build()
        .map_err(Into::into)
}

#[allow(clippy::too_many_lines)]
pub async fn get_standup_data(
    access_token: &str,
    team_id: &str,
    user_id: &str,
) -> Result<StandupData> {
    let client = create_linear_client(access_token).await?;

    // Execute all queries in parallel for better performance
    let (
        viewer_and_team,
        active_cycle,
        previous_cycle,
        in_progress,
        recently_updated,
        backlog,
        with_blockers,
    ) = tokio::try_join!(
        // Query 1: Viewer and Team
        async {
            let response = post_graphql::<StandupViewerAndTeam, _>(
                &client,
                "https://api.linear.app/graphql",
                standup_viewer_and_team::Variables {
                    team_id: team_id.to_string(),
                },
            )
            .await?;

            if let Some(errors) = response.errors {
                return Err(cja::color_eyre::eyre::eyre!(
                    "GraphQL errors in viewer_and_team: {:?}",
                    errors
                ));
            }

            response.data.ok_or_else(|| {
                cja::color_eyre::eyre::eyre!("No data returned from viewer_and_team query")
            })
        },
        // Query 2: Active Cycle
        async {
            let response = post_graphql::<StandupActiveCycle, _>(
                &client,
                "https://api.linear.app/graphql",
                standup_active_cycle::Variables {
                    team_id: team_id.to_string(),
                },
            )
            .await?;

            if let Some(errors) = response.errors {
                return Err(cja::color_eyre::eyre::eyre!(
                    "GraphQL errors in active_cycle: {:?}",
                    errors
                ));
            }

            response.data.ok_or_else(|| {
                cja::color_eyre::eyre::eyre!("No data returned from active_cycle query")
            })
        },
        // Query 3: Previous Cycle
        async {
            let response = post_graphql::<StandupPreviousCycle, _>(
                &client,
                "https://api.linear.app/graphql",
                standup_previous_cycle::Variables {
                    team_id: team_id.to_string(),
                    user_id: user_id.to_string(),
                },
            )
            .await?;

            if let Some(errors) = response.errors {
                return Err(cja::color_eyre::eyre::eyre!(
                    "GraphQL errors in previous_cycle: {:?}",
                    errors
                ));
            }

            response.data.ok_or_else(|| {
                cja::color_eyre::eyre::eyre!("No data returned from previous_cycle query")
            })
        },
        // Query 4: In Progress Issues
        async {
            let response = post_graphql::<StandupInProgressIssues, _>(
                &client,
                "https://api.linear.app/graphql",
                standup_in_progress_issues::Variables {
                    user_id: user_id.to_string(),
                },
            )
            .await?;

            if let Some(errors) = response.errors {
                return Err(cja::color_eyre::eyre::eyre!(
                    "GraphQL errors in in_progress_issues: {:?}",
                    errors
                ));
            }

            response.data.ok_or_else(|| {
                cja::color_eyre::eyre::eyre!("No data returned from in_progress_issues query")
            })
        },
        // Query 5: Recently Updated Issues
        async {
            let response = post_graphql::<StandupRecentlyUpdatedIssues, _>(
                &client,
                "https://api.linear.app/graphql",
                standup_recently_updated_issues::Variables {
                    user_id: user_id.to_string(),
                },
            )
            .await?;

            if let Some(errors) = response.errors {
                return Err(cja::color_eyre::eyre::eyre!(
                    "GraphQL errors in recently_updated_issues: {:?}",
                    errors
                ));
            }

            response.data.ok_or_else(|| {
                cja::color_eyre::eyre::eyre!("No data returned from recently_updated_issues query")
            })
        },
        // Query 6: Backlog Issues
        async {
            let response = post_graphql::<StandupBacklogIssues, _>(
                &client,
                "https://api.linear.app/graphql",
                standup_backlog_issues::Variables {
                    team_id: team_id.to_string(),
                },
            )
            .await?;

            if let Some(errors) = response.errors {
                return Err(cja::color_eyre::eyre::eyre!(
                    "GraphQL errors in backlog_issues: {:?}",
                    errors
                ));
            }

            response.data.ok_or_else(|| {
                cja::color_eyre::eyre::eyre!("No data returned from backlog_issues query")
            })
        },
        // Query 7: Issues with Blockers
        async {
            let response = post_graphql::<StandupIssuesWithBlockers, _>(
                &client,
                "https://api.linear.app/graphql",
                standup_issues_with_blockers::Variables {
                    user_id: user_id.to_string(),
                },
            )
            .await?;

            if let Some(errors) = response.errors {
                return Err(cja::color_eyre::eyre::eyre!(
                    "GraphQL errors in issues_with_blockers: {:?}",
                    errors
                ));
            }

            response.data.ok_or_else(|| {
                cja::color_eyre::eyre::eyre!("No data returned from issues_with_blockers query")
            })
        },
    )?;

    // Combine all the data
    Ok(StandupData {
        viewer: Some(viewer_and_team.viewer),
        team: Some(viewer_and_team.team),
        active_cycle: active_cycle.team.active_cycle,
        previous_cycle: previous_cycle.team.previous_cycle.nodes,
        in_progress_issues: Some(in_progress.in_progress_issues.nodes),
        recently_updated_issues: Some(recently_updated.recently_updated_issues.nodes),
        backlog_issues: Some(backlog.backlog_issues.nodes),
        issues_with_blockers: Some(with_blockers.issues_with_blockers.nodes),
    })
}
