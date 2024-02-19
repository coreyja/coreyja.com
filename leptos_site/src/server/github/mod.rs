use miette::{IntoDiagnostic, Result};
use tracing::instrument;

pub mod sponsors;

#[derive(Debug, Clone)]
pub(crate) struct GithubConfig {
    pub(crate) app_id: u64,
    pub(crate) client_id: String,
    pub(crate) client_secret: String,
    pub(crate) pat: String,
    pub(crate) app_private_key: String,
}

impl GithubConfig {
    #[instrument(name = "GithubConfig::from_env")]
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

#[derive(Debug, Clone)]
pub struct GithubLink {
    pub github_link_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub external_github_login: String,
    pub external_github_id: String,
    pub encrypted_access_token: Vec<u8>,
    pub encrypted_refresh_token: Vec<u8>,
    pub access_token_expires_at: chrono::DateTime<chrono::Utc>,
    pub refresh_token_expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
