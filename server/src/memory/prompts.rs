use color_eyre::Result;
use sqlx::PgPool;

use super::blocks::MemoryBlock;

#[derive(Debug)]
pub struct PromptGenerator;

impl PromptGenerator {
    pub fn new() -> Self {
        Self
    }

    pub async fn generate_system_prompt(pool: &PgPool, is_discord_context: bool) -> Result<String> {
        // Base instructions (always included)
        let mut system_content = String::from(
            "You are an AI assistant with the following capabilities and constraints:\n\
            - Be helpful, accurate, and thoughtful in your responses\n\
            - Follow user instructions carefully\n\
            - Admit when you're unsure rather than making things up\n\
            - Use the tools available to you when appropriate\n",
        );

        // Add persona if available
        let persona = MemoryBlock::get_persona(pool).await?;
        if let Some(persona_block) = persona {
            system_content.push('\n');
            system_content.push_str(&persona_block.content);
            system_content.push('\n');
        }

        // Add context-specific instructions
        if is_discord_context {
            system_content.push_str(
                "\nContext-specific instructions for Discord:\n\
                - Keep responses under 2000 characters (Discord's limit)\n\
                - Use Discord markdown formatting when appropriate\n\
                - Be conversational and friendly\n\
                - You can react to messages using emojis when appropriate\n\
                - Each message shows the Message ID that you can use to react to specific messages\n\
                - You can list available custom server emojis using the list_server_emojis tool"
            );
        }

        Ok(system_content)
    }

    pub fn base_instructions() -> &'static str {
        "You are an AI assistant with the following capabilities and constraints:\n\
        - Be helpful, accurate, and thoughtful in your responses\n\
        - Follow user instructions carefully\n\
        - Admit when you're unsure rather than making things up\n\
        - Use the tools available to you when appropriate\n"
    }

    pub fn discord_instructions() -> &'static str {
        "\nContext-specific instructions for Discord:\n\
        - Keep responses under 2000 characters (Discord's limit)\n\
        - Use Discord markdown formatting when appropriate\n\
        - Be conversational and friendly\n\
        - You can react to messages using emojis when appropriate\n\
        - Each message shows the Message ID that you can use to react to specific messages\n\
        - You can list available custom server emojis using the list_server_emojis tool"
    }
}
