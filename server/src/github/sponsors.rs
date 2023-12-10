use chrono::DateTime as ChronoDateTime;
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};

use self::get_sponsors::Variables;

type DateTime = ChronoDateTime<chrono::Utc>;

// The paths are relative to the directory where your `Cargo.toml` is located.
// Both json and the GraphQL schema language are supported as sources for the schema
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/github/schema.graphql",
    query_path = "src/github/get_sponsors.graphql",
    response_derives = "Debug"
)]
pub struct GetSponsors;

pub async fn get_sponsors(access_token: &str) -> Result<Vec<Sponsor>> {
    let client = reqwest::Client::builder()
        .user_agent("github.com/coreyja/coreyja.com")
        .default_headers(
            std::iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", access_token))
                    .into_diagnostic()?,
            ))
            .collect(),
        )
        .build()
        .into_diagnostic()?;

    let response_body =
        post_graphql::<GetSponsors, _>(&client, "https://api.github.com/graphql", Variables {})
            .await
            .unwrap();

    let response = response_body
        .data
        .ok_or_else(|| miette::miette!("There were no sponsors"))?;

    Ok(response
        .viewer
        .sponsorships_as_maintainer
        .edges
        .ok_or_else(|| miette::miette!("There were no edges"))?
        .into_iter()
        .filter_map(|edge| {
            let node = edge?.node?;
            let (sponsor_type, login, name, id) = match &node.sponsor_entity? {
                get_sponsors::GetSponsorsViewerSponsorshipsAsMaintainerEdgesNodeSponsorEntity::Organization(o) => (SponsorType::Organization, o.login.clone(), o.name.clone(), o.id.clone()),
                get_sponsors::GetSponsorsViewerSponsorshipsAsMaintainerEdgesNodeSponsorEntity::User(u) => (SponsorType::User, u.login.clone(), u.name.clone(), u.id.clone()),
            };

            let created_at = node.created_at;
            let is_active = node.is_active;
            let is_one_time_payment = node.is_one_time_payment;

            let tier = node.tier.map(|t| Tier {
                name: t.name,
                monthly_price_in_cents: t.monthly_price_in_cents,
            });

            let privacy_level = match &node.privacy_level {
                get_sponsors::SponsorshipPrivacy::PRIVATE => "private",
                get_sponsors::SponsorshipPrivacy::PUBLIC => "public",
                get_sponsors::SponsorshipPrivacy::Other(x) => x,
            }.to_owned();

            Some(Sponsor {
                sponsor_type,
                login,
                name,
                id,
                created_at,
                is_active,
                is_one_time_payment,
                tier,
                privacy_level,
            })
        })
        .collect())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sponsor {
    sponsor_type: SponsorType,
    id: String,
    login: String,
    name: Option<String>,
    created_at: DateTime,
    is_active: bool,
    is_one_time_payment: bool,
    tier: Option<Tier>,
    privacy_level: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tier {
    name: String,
    monthly_price_in_cents: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SponsorType {
    User,
    Organization,
}
