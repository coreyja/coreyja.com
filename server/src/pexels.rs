use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub(crate) struct PexelsResponse {
    pub page: i32,
    pub per_page: i32,
    pub photos: Vec<Photo>,
    pub total_results: i32,
    pub next_page: Option<String>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Photo {
    pub id: i32,
    pub width: i32,
    pub height: i32,
    pub url: String,
    pub photographer: String,
    pub photographer_url: String,
    pub src: PhotoSources,
}

#[derive(Deserialize, Debug)]
pub(crate) struct PhotoSources {
    pub original: String,
    pub large: String,
    pub medium: String,
    pub small: String,
    pub thumbnail: String,
}

pub(crate) async fn fetch_user_photos(
    username: &str,
    api_key: &str,
    page: i32,
) -> cja::Result<PexelsResponse> {
    let client = reqwest::Client::new();
    let url = format!("https://api.pexels.com/v1/users/{username}/photos?page={page}&per_page=20");

    let response = client
        .get(&url)
        .header("Authorization", api_key)
        .send()
        .await?
        .text()
        .await?;

    dbg!(&response);

    let response: PexelsResponse = serde_json::from_str(&response)?;

    Ok(response)
}

#[derive(Debug, Clone)]
pub(crate) struct PexelsConfig {
    pub api_key: String,
}

impl PexelsConfig {
    pub(crate) fn from_env() -> cja::Result<Self> {
        Ok(Self {
            api_key: std::env::var("PEXELS_API_KEY")?,
        })
    }
}
