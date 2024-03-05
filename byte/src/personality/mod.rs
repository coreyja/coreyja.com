use openai::chat::ChatMessage;

const BASE_PROMPT: &str = include_str!("base_prompt.txt");

#[must_use] pub fn base() -> ChatMessage {
    ChatMessage {
        role: openai::chat::ChatRole::System,
        content: BASE_PROMPT.to_string(),
    }
}

#[must_use] pub fn respond_to_twitch_chat_prompt() -> ChatMessage {
    ChatMessage {
        role: openai::chat::ChatRole::System,
        content: indoc::indoc! {"
            The following is a chat message from Twitch chat.
            You should respond to it as best you can while remaining brief.
        "}
        .to_string(),
    }
}
