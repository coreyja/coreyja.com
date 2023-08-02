use crate::*;

#[derive(Serialize, Deserialize)]
struct ChatCompletionBody {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: Option<i64>,
    max_tokens: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    User,
    Assistant,
    Function,
}

#[derive(Serialize, Deserialize)]
struct CompletionChoice {
    index: i64,
    message: ChatMessage,
    finish_reason: String,
}

#[derive(Serialize, Deserialize)]
struct CompletionUsage {
    completion_tokens: i64,
    prompt_tokens: i64,
    total_tokens: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CompletionResponse {
    choices: Vec<CompletionChoice>,
    created: i64,
    id: String,
    model: String,
    object: String,
    usage: CompletionUsage,
}

pub async fn complete_chat(
    config: &OpenAiConfig,
    model: &str,
    messages: Vec<ChatMessage>,
) -> Result<ChatMessage> {
    let client = reqwest::Client::new();

    let body = ChatCompletionBody {
        model: model.to_string(),
        messages,
        temperature: None,
        max_tokens: None,
    };
    let res = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(&config.api_key)
        .json(&body)
        .send()
        .await
        .into_diagnostic()?;
    if res.status() != 200 {
        println!("Status: {}", res.status());
        println!(
            "Body: {:#?}",
            res.json::<serde_json::Value>().await.unwrap()
        );

        return Err(miette::miette!("Failed to complete chat"));
    }
    let body = res.json::<CompletionResponse>().await.into_diagnostic()?;

    let msg = body.choices.into_iter().next().unwrap().message;

    Ok(msg)
}
