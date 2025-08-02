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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::blocks::MemoryBlockType;

    #[test]
    fn test_prompt_generator_new() {
        let generator = PromptGenerator::new();
        // Just verify it can be created
        let _ = format!("{generator:?}");
    }

    #[test]
    fn test_base_instructions() {
        let instructions = PromptGenerator::base_instructions();
        assert!(instructions.contains("AI assistant"));
        assert!(instructions.contains("helpful"));
        assert!(instructions.contains("accurate"));
        assert!(instructions.contains("thoughtful"));
        assert!(instructions.contains("Follow user instructions"));
        assert!(instructions.contains("Admit when you're unsure"));
        assert!(instructions.contains("Use the tools available"));
    }

    #[test]
    fn test_discord_instructions() {
        let instructions = PromptGenerator::discord_instructions();
        assert!(instructions.contains("Discord"));
        assert!(instructions.contains("2000 characters"));
        assert!(instructions.contains("Discord markdown"));
        assert!(instructions.contains("conversational"));
        assert!(instructions.contains("emojis"));
        assert!(instructions.contains("Message ID"));
        assert!(instructions.contains("list_server_emojis"));
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_generate_system_prompt_without_persona(pool: PgPool) {
        // Generate prompt without persona
        let prompt = PromptGenerator::generate_system_prompt(&pool, false)
            .await
            .unwrap();

        // Should contain base instructions
        assert!(prompt.contains("AI assistant"));
        assert!(prompt.contains("helpful"));

        // Should not contain Discord instructions
        assert!(!prompt.contains("Discord"));
        assert!(!prompt.contains("2000 characters"));
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_generate_system_prompt_with_persona(pool: PgPool) {
        // Create a persona
        let persona_content =
            "I am a friendly and knowledgeable assistant who loves to help with coding.";
        MemoryBlock::create(&pool, MemoryBlockType::Persona, persona_content.to_string())
            .await
            .unwrap();

        // Generate prompt with persona
        let prompt = PromptGenerator::generate_system_prompt(&pool, false)
            .await
            .unwrap();

        // Should contain base instructions
        assert!(prompt.contains("AI assistant"));

        // Should contain persona content
        assert!(prompt.contains(persona_content));

        // Should not contain Discord instructions
        assert!(!prompt.contains("Discord"));
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_generate_system_prompt_discord_context(pool: PgPool) {
        // Generate prompt for Discord context
        let prompt = PromptGenerator::generate_system_prompt(&pool, true)
            .await
            .unwrap();

        // Should contain base instructions
        assert!(prompt.contains("AI assistant"));

        // Should contain Discord instructions
        assert!(prompt.contains("Discord"));
        assert!(prompt.contains("2000 characters"));
        assert!(prompt.contains("Discord markdown"));
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_generate_system_prompt_discord_with_persona(pool: PgPool) {
        // Create a persona
        let persona_content = "I am Discord bot with a playful personality.";
        MemoryBlock::create(&pool, MemoryBlockType::Persona, persona_content.to_string())
            .await
            .unwrap();

        // Generate prompt for Discord context
        let prompt = PromptGenerator::generate_system_prompt(&pool, true)
            .await
            .unwrap();

        // Should contain all three: base, persona, and Discord instructions
        assert!(prompt.contains("AI assistant"));
        assert!(prompt.contains(persona_content));
        assert!(prompt.contains("Discord"));
        assert!(prompt.contains("2000 characters"));
    }
}
