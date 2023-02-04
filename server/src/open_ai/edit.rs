use crate::*;

#[derive(Serialize, Deserialize)]
struct EditRequest {
    input: String,
    instruction: String,
    model: String,
}

const EDIT_MODEL: &str = "text-davinci-edit-001";

impl EditRequest {
    fn new(input: String, instruction: String) -> Self {
        Self {
            input,
            instruction,
            model: EDIT_MODEL.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct EditChoice {
    index: i64,
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct EditUsage {
    completion_tokens: i64,
    prompt_tokens: i64,
    total_tokens: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct EditResponse {
    choices: Vec<EditChoice>,
    created: i64,
    object: String,
    usage: EditUsage,
}

pub(crate) async fn edit_prompt(
    config: &OpenAiConfig,
    prompt: impl Into<String>,
    instructions: impl Into<String>,
) -> Result<String> {
    let client = reqwest::Client::new();

    let body = EditRequest::new(prompt.into(), instructions.into());
    let res = client
        .post("https://api.openai.com/v1/edits")
        .bearer_auth(&config.api_key)
        .json::<EditRequest>(&body)
        .send()
        .await?;
    let body = res.json::<EditResponse>().await?;

    dbg!(&body);

    let text = body.choices.into_iter().next().unwrap().text;
    Ok(text)
}
