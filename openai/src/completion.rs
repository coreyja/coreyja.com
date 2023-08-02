use crate::*;

const COMPLETION_MODEL: &str = "text-davinci-003";

#[derive(Serialize, Deserialize)]
struct CompletionBody {
    max_tokens: Option<i64>,
    model: String,
    prompt: String,
    temperature: Option<i64>,
}

impl CompletionBody {
    fn new(prompt: String) -> Self {
        Self {
            max_tokens: Some(2048),
            model: COMPLETION_MODEL.to_string(),
            prompt,
            temperature: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CompletionChoice {
    index: i64,
    text: String,
}

#[derive(Serialize, Deserialize)]
struct CompletionUsage {
    completion_tokens: i64,
    prompt_tokens: i64,
    total_tokens: i64,
}

#[derive(Serialize, Deserialize)]
struct CompletionResponse {
    choices: Vec<CompletionChoice>,
    created: i64,
    id: String,
    model: String,
    object: String,
    usage: CompletionUsage,
}

pub async fn complete_prompt(config: &OpenAiConfig, prompt: impl Into<String>) -> Result<String> {
    let client = reqwest::Client::new();

    let body = CompletionBody::new(prompt.into());
    let res = client
        .post("https://api.openai.com/v1/completions")
        .bearer_auth(&config.api_key)
        .json(&body)
        .send()
        .await
        .into_diagnostic()?;
    let body = res.json::<CompletionResponse>().await.into_diagnostic()?;

    let text = body.choices.into_iter().next().unwrap().text;
    Ok(text)
}
