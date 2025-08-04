use cja::Result;
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use reqwest::Client;

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
