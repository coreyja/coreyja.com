use chrono::DateTime as ChronoDateTime;
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, QueryBuilder};
use uuid::Uuid;

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

    let response =
        post_graphql::<GetSponsors, _>(&client, "https://api.github.com/graphql", Variables {})
            .await
            .unwrap();

    let response_body = response.data.ok_or_else(|| {
        miette::miette!(
            "No data was returned in the query for Sponsors: {:?}",
            &response.errors
        )
    })?;

    Ok(response_body
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

impl SponsorType {
    fn as_db(&self) -> &'static str {
        match &self {
            SponsorType::User => "user",
            SponsorType::Organization => "organization",
        }
    }
}

async fn insert_sponsors(sponsors: &[Sponsor], pool: &Pool<Postgres>) -> Result<()> {
    let mut query_builder = QueryBuilder::new(
        "
    INSERT INTO GithubSponsors (
        github_sponsor_id,
        sponsor_type,
        github_id,
        github_login,
        sponsored_at,
        is_active,
        is_one_time_payment,
        tier_name,
        amount_cents,
        privacy_level
    )",
    );

    query_builder.push_values(sponsors, |mut b, sponsor| {
        b.push_bind(Uuid::new_v4())
            .push_bind(sponsor.sponsor_type.as_db())
            .push_bind(sponsor.id.clone())
            .push_bind(sponsor.login.clone())
            .push_bind(sponsor.created_at)
            .push_bind(sponsor.is_active)
            .push_bind(sponsor.is_one_time_payment)
            .push_bind(sponsor.tier.as_ref().map(|t| t.name.to_owned()))
            .push_bind(sponsor.tier.as_ref().map(|t| t.monthly_price_in_cents))
            .push_bind(sponsor.privacy_level.to_owned());
    });
    query_builder.push(
        "ON CONFLICT (github_id) DO UPDATE SET (
        sponsor_type,
        github_login,
        sponsored_at,
        is_active,
        is_one_time_payment,
        tier_name,
        amount_cents,
        privacy_level
    ) =
    (
        EXCLUDED.sponsor_type,
        EXCLUDED.github_login,
        EXCLUDED.sponsored_at,
        EXCLUDED.is_active,
        EXCLUDED.is_one_time_payment,
        EXCLUDED.tier_name,
        EXCLUDED.amount_cents,
        EXCLUDED.privacy_level
    )",
    );

    let query = query_builder.build();

    query.execute(pool).await.into_diagnostic()?;

    Ok(())
}

pub async fn refresh_db(app_state: &crate::AppState) -> Result<()> {
    let sponsors = get_sponsors(&app_state.github.pat).await?;

    insert_sponsors(&sponsors, &app_state.db).await?;

    sqlx::query!(
        r#"
        DELETE FROM GithubSponsors
        WHERE github_id not in (Select * from UNNEST($1::text[]))
        "#,
        &sponsors.iter().map(|s| s.id.clone()).collect::<Vec<_>>()
    )
    .execute(&app_state.db)
    .await
    .into_diagnostic()?;

    set_last_refresh_at(app_state, "github_sponsors").await?;

    Ok(())
}

pub async fn set_last_refresh_at(
    app_state: &crate::state::AppState,
    key: &str,
) -> Result<(), miette::ErrReport> {
    sqlx::query!(
        "
        INSERT INTO LastRefreshAts (
            key,
            last_refresh_at
        ) VALUES (
            $1,
            NOW()
        ) ON CONFLICT (key) DO UPDATE SET
            last_refresh_at = excluded.last_refresh_at
        ",
        key
    )
    .execute(&app_state.db)
    .await
    .into_diagnostic()?;

    Ok(())
}
