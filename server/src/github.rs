use std::collections::HashMap;

use crate::*;

use miette::{IntoDiagnostic, Result};
use tracing::instrument;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct GithubConfig {
    pub(crate) app_id: u64,
    pub(crate) client_id: String,
    pub(crate) client_secret: String,
    pub(crate) pat: String,
    pub(crate) app_private_key: String,
}

impl GithubConfig {
    #[instrument]
    pub(crate) fn from_env() -> Result<Self> {
        Ok(Self {
            app_id: std::env::var("GITHUB_APP_ID")
                .into_diagnostic()?
                .parse()
                .into_diagnostic()?,
            client_id: std::env::var("GITHUB_APP_CLIENT_ID").into_diagnostic()?,
            client_secret: std::env::var("GITHUB_APP_CLIENT_SECRET").into_diagnostic()?,
            pat: std::env::var("GITHUB_PERSONAL_ACCESS_TOKEN").into_diagnostic()?,
            app_private_key: std::env::var("GITHUB_APP_PRIVATE_KEY").into_diagnostic()?,
        })
    }
}

pub async fn get_sponsorships(access_token: &str) -> Result<Vec<Sponsor>> {
    let endpoint = "https://api.github.com/graphql";
    let query = r#"
    {
        viewer {
          sponsorshipsAsMaintainer(first: 100) {
            edges {
              node {
                sponsorEntity {
                  ... on User {
                    login
                    name
                    id
                  }
                  ... on Organization {
                    login
                    name
                    id
                  }
                }
                createdAt
                isActive
                isOneTimePayment
                tier {
                  name
                  monthlyPriceInCents
                }
                privacyLevel
              }
            }
          }
        }
      }
   "#;
    let mut headers = HashMap::new();
    headers.insert("Authorization", format!("Bearer {}", access_token));
    headers.insert("User-Agent", "github.com/coreyja/coreyja.com".to_string());
    headers.insert("Accept", "application/vnd.github.v3+json".to_string());

    let client = gql_client::Client::new_with_headers(endpoint, headers);
    let data = client
        .query::<serde_json::Value>(query)
        .await
        .map_err(|e| miette::miette!("Failed to query github : {:#?}", e))?;

    let sponsors = data.ok_or_else(|| miette::miette!("There were no sponsors"))?;

    let edges = &sponsors["viewer"]["sponsorshipsAsMaintainer"]["edges"];
    dbg!(&edges);
    let sponsors = edges
        .as_array()
        .ok_or_else(|| miette::miette!("The edges wasn't an array"))?
        .iter()
        .map(|edge| {
            let node = edge["node"]["sponsorEntity"].clone();
            let login = node["login"]
                .as_str()
                .ok_or_else(|| miette::miette!("Login was not a string"))?
                .to_string();
            let id = node["id"]
                .as_str()
                .ok_or_else(|| miette::miette!("Id was not a string"))?
                .to_string();
            Ok(Sponsor { login, id })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(sponsors)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sponsor {
    id: String,
    login: String,
}

pub(crate) async fn generate_server_token(
    config: &GithubConfig,
    installation_id: String,
) -> Result<String> {
    #[derive(Debug, serde::Serialize)]
    struct Claims {
        iss: String,
        iat: i64,
        exp: i64,
    }
    let key = jsonwebtoken::EncodingKey::from_rsa_pem(config.app_private_key.as_bytes())
        .into_diagnostic()?;

    let jwt = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256),
        &Claims {
            iss: config.app_id.to_string(),
            iat: (chrono::Utc::now() - chrono::Duration::seconds(10)).timestamp(),
            exp: (chrono::Utc::now() + chrono::Duration::minutes(8)).timestamp(),
        },
        &key,
    )
    .into_diagnostic()?;

    let url = format!("https://api.github.com/app/installations/{installation_id}/access_tokens");
    dbg!(&url);
    let client = reqwest::Client::new();
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "Accept",
        "application/vnd.github+json".parse().into_diagnostic()?,
    );
    headers.insert(
        "Authorization",
        format!("Bearer {jwt}").parse().into_diagnostic()?,
    );
    headers.insert(
        "X-GitHub-Api-Version",
        "2022-11-28".parse().into_diagnostic()?,
    );
    headers.insert(
        "User-Agent",
        "github.com/coreyja/coreyja.com".parse().into_diagnostic()?,
    );

    let token_response = client
        .post(&url)
        .headers(headers)
        .send()
        .await
        .into_diagnostic()?;
    dbg!(&token_response);

    let token_response = token_response
        .json::<serde_json::Value>()
        .await
        .into_diagnostic()?;

    dbg!(&token_response);

    let token = token_response["token"]
        .as_str()
        .ok_or_else(|| miette::miette!("Token was not a string"))?;

    Ok(token.to_string())
}
