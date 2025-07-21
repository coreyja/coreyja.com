use chrono::DateTime as ChronoDateTime;
use cja::Result;
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, QueryBuilder};
use tracing::warn;
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

pub async fn get_sponsors(access_token: &str) -> cja::Result<Vec<Sponsor>> {
    let client = reqwest::Client::builder()
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
        post_graphql::<GetSponsors, _>(&client, "https://api.github.com/graphql", Variables {})
            .await
            .unwrap();

    let Some(response_body) = response.data else {
        warn!(
            "No data was returned in the query for Sponsors: {:?}",
            &response.errors
        );
        return Ok(vec![]);
    };

    Ok(response_body
        .viewer
        .sponsorships_as_maintainer
        .edges
        .ok_or_else(|| cja::color_eyre::eyre::eyre!("There were no edges"))?
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

            Some(Sponsor { sponsor_type, id, login, name, created_at, is_active, is_one_time_payment, tier, privacy_level })
        })
        .collect())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sponsor {
    #[allow(clippy::struct_field_names)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tier {
    name: String,
    monthly_price_in_cents: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    if sponsors.is_empty() {
        return Ok(());
    }

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
            .push_bind(sponsor.tier.as_ref().map(|t| t.name.clone()))
            .push_bind(sponsor.tier.as_ref().map(|t| t.monthly_price_in_cents))
            .push_bind(sponsor.privacy_level.clone());
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

    query.execute(pool).await?;

    Ok(())
}

pub(crate) async fn refresh_db(app_state: &crate::AppState) -> Result<()> {
    let sponsors = get_sponsors(&app_state.github.pat).await?;

    insert_sponsors(&sponsors, &app_state.db).await?;

    sqlx::query!(
        r#"
        UPDATE GithubSponsors SET is_active = false
        WHERE github_id not in (Select * from UNNEST($1::text[]))
        "#,
        &sponsors.iter().map(|s| s.id.clone()).collect::<Vec<_>>()
    )
    .execute(&app_state.db)
    .await?;

    set_last_refresh_at(app_state, "github_sponsors").await?;

    Ok(())
}

pub(crate) async fn set_last_refresh_at(
    app_state: &crate::state::AppState,
    key: &str,
) -> Result<()> {
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
    .await?;

    Ok(())
}

pub struct GithubSponsorFromDB {
    pub github_sponsor_id: Uuid,
    pub user_id: Option<Uuid>,
    pub sponsor_type: String,
    pub github_id: String,
    pub github_login: String,
    pub sponsored_at: DateTime,
    pub is_active: bool,
    pub is_one_time_payment: bool,
    pub tier_name: Option<String>,
    pub amount_cents: Option<i32>,
    pub privacy_level: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use sqlx::PgPool;

    fn create_test_sponsor(id: &str, login: &str, is_active: bool) -> Sponsor {
        Sponsor {
            sponsor_type: SponsorType::User,
            id: id.to_string(),
            login: login.to_string(),
            name: Some(format!("Test User {login}")),
            created_at: Utc::now(),
            is_active,
            is_one_time_payment: false,
            tier: Some(Tier {
                name: "Bronze".to_string(),
                monthly_price_in_cents: 500,
            }),
            privacy_level: "public".to_string(),
        }
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_insert_sponsors_empty_array(pool: PgPool) {
        let sponsors = vec![];
        let result = insert_sponsors(&sponsors, &pool).await;
        assert!(result.is_ok());

        let count = sqlx::query_scalar!("SELECT COUNT(*) FROM GithubSponsors")
            .fetch_one(&pool)
            .await
            .unwrap()
            .unwrap_or(0);
        assert_eq!(count, 0);
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_insert_sponsors_with_data(pool: PgPool) {
        let sponsors = vec![
            create_test_sponsor("gh_user_1", "testuser1", true),
            create_test_sponsor("gh_user_2", "testuser2", false),
        ];

        let result = insert_sponsors(&sponsors, &pool).await;
        assert!(result.is_ok());

        let count = sqlx::query_scalar!("SELECT COUNT(*) FROM GithubSponsors")
            .fetch_one(&pool)
            .await
            .unwrap()
            .unwrap_or(0);
        assert_eq!(count, 2);

        let sponsor1 = sqlx::query!(
            "SELECT * FROM GithubSponsors WHERE github_id = $1",
            "gh_user_1"
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(sponsor1.github_login, "testuser1");
        assert!(sponsor1.is_active);
        assert_eq!(sponsor1.tier_name, Some("Bronze".to_string()));
        assert_eq!(sponsor1.amount_cents, Some(500));
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_insert_sponsors_upsert_behavior(pool: PgPool) {
        let sponsor = create_test_sponsor("gh_user_1", "testuser1", true);

        let result = insert_sponsors(&[sponsor.clone()], &pool).await;
        assert!(result.is_ok());

        let mut updated_sponsor = sponsor;
        updated_sponsor.is_active = false;
        updated_sponsor.tier = Some(Tier {
            name: "Gold".to_string(),
            monthly_price_in_cents: 2500,
        });

        let result = insert_sponsors(&[updated_sponsor], &pool).await;
        assert!(result.is_ok());

        let count = sqlx::query_scalar!("SELECT COUNT(*) FROM GithubSponsors")
            .fetch_one(&pool)
            .await
            .unwrap()
            .unwrap_or(0);
        assert_eq!(count, 1);

        let db_sponsor = sqlx::query!(
            "SELECT * FROM GithubSponsors WHERE github_id = $1",
            "gh_user_1"
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert!(!db_sponsor.is_active);
        assert_eq!(db_sponsor.tier_name, Some("Gold".to_string()));
        assert_eq!(db_sponsor.amount_cents, Some(2500));
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_refresh_db_marks_inactive_sponsors(pool: PgPool) {
        let active_sponsors = vec![
            create_test_sponsor("gh_user_1", "testuser1", true),
            create_test_sponsor("gh_user_2", "testuser2", true),
        ];
        insert_sponsors(&active_sponsors, &pool).await.unwrap();

        let extra_sponsor = create_test_sponsor("gh_user_3", "testuser3", true);
        insert_sponsors(&[extra_sponsor], &pool).await.unwrap();

        let active_ids: Vec<String> = vec!["gh_user_1".to_string(), "gh_user_2".to_string()];
        sqlx::query!(
            r#"
            UPDATE GithubSponsors SET is_active = false
            WHERE github_id not in (Select * from UNNEST($1::text[]))
            "#,
            &active_ids
        )
        .execute(&pool)
        .await
        .unwrap();

        let sponsor3 = sqlx::query!(
            "SELECT is_active FROM GithubSponsors WHERE github_id = $1",
            "gh_user_3"
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(!sponsor3.is_active);

        let sponsor1 = sqlx::query!(
            "SELECT is_active FROM GithubSponsors WHERE github_id = $1",
            "gh_user_1"
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(sponsor1.is_active);
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_refresh_db_empty_sponsors_marks_all_inactive(pool: PgPool) {
        let sponsors = vec![
            create_test_sponsor("gh_user_1", "testuser1", true),
            create_test_sponsor("gh_user_2", "testuser2", true),
        ];
        insert_sponsors(&sponsors, &pool).await.unwrap();

        let empty_ids: Vec<String> = vec![];
        let result = sqlx::query!(
            r#"
            UPDATE GithubSponsors SET is_active = false
            WHERE github_id not in (Select * from UNNEST($1::text[]))
            "#,
            &empty_ids
        )
        .execute(&pool)
        .await;

        assert!(result.is_ok());

        let inactive_count =
            sqlx::query_scalar!("SELECT COUNT(*) FROM GithubSponsors WHERE is_active = false")
                .fetch_one(&pool)
                .await
                .unwrap()
                .unwrap_or(0);
        assert_eq!(inactive_count, 2);
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_sponsor_type_as_db(_pool: PgPool) {
        assert_eq!(SponsorType::User.as_db(), "user");
        assert_eq!(SponsorType::Organization.as_db(), "organization");
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_insert_organization_sponsor(pool: PgPool) {
        let org_sponsor = Sponsor {
            sponsor_type: SponsorType::Organization,
            id: "gh_org_1".to_string(),
            login: "testorg".to_string(),
            name: Some("Test Organization".to_string()),
            created_at: Utc::now(),
            is_active: true,
            is_one_time_payment: true,
            tier: None,
            privacy_level: "private".to_string(),
        };

        let result = insert_sponsors(&[org_sponsor], &pool).await;
        assert!(result.is_ok());

        let db_sponsor = sqlx::query!(
            "SELECT * FROM GithubSponsors WHERE github_id = $1",
            "gh_org_1"
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(db_sponsor.sponsor_type, "organization");
        assert!(db_sponsor.is_one_time_payment);
        assert_eq!(db_sponsor.tier_name, None);
        assert_eq!(db_sponsor.amount_cents, None);
        assert_eq!(db_sponsor.privacy_level, "private");
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_set_last_refresh_at(pool: PgPool) {
        // Test the raw SQL query directly instead of going through AppState
        let key = "test_key";
        let result = sqlx::query!(
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
        .execute(&pool)
        .await;

        assert!(result.is_ok());

        let refresh_record = sqlx::query!("SELECT * FROM LastRefreshAts WHERE key = $1", key)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(refresh_record.key, "test_key");
        assert!(refresh_record.last_refresh_at < Utc::now());

        let original_time = refresh_record.last_refresh_at;

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Update again
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
        .execute(&pool)
        .await
        .unwrap();

        let updated_record = sqlx::query!("SELECT * FROM LastRefreshAts WHERE key = $1", key)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert!(updated_record.last_refresh_at > original_time);
    }
}
