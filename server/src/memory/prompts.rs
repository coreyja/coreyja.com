use color_eyre::Result;
use db::agentic_threads::{Thread, ThreadMode, ThreadType};
use sqlx::PgPool;
use std::fmt::Write;

use super::blocks::MemoryBlock;

#[derive(Debug)]
pub struct PromptGenerator;

impl PromptGenerator {
    pub fn new() -> Self {
        Self
    }

    pub async fn generate_system_prompt(pool: &PgPool, thread: &Thread) -> Result<String> {
        // Base instructions (always included)
        let mut system_content = Self::base_instructions().to_string();

        // Add thread goal
        write!(system_content, "\nCurrent goal: {}\n", thread.goal)?;

        // Add mode-specific instructions
        let mode_instructions = Self::generate_mode_instructions(&thread.mode);
        if !mode_instructions.is_empty() {
            system_content.push_str("\n--- MODE-SPECIFIC INSTRUCTIONS ---\n");
            system_content.push_str(mode_instructions);
            system_content.push_str("\n--- END MODE-SPECIFIC INSTRUCTIONS ---\n");
        }

        // Add persona if available
        let persona = MemoryBlock::get_persona(pool).await?;
        if let Some(persona_block) = persona {
            system_content.push_str("\n--- PERSONA MEMORY BLOCK ---\n");
            system_content.push_str(&persona_block.content);
            system_content.push_str("\n--- END PERSONA MEMORY BLOCK ---\n");
        }

        // Add context-specific instructions for Discord
        if thread.thread_type == ThreadType::Interactive {
            system_content.push_str(Self::discord_instructions());
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

    pub fn generate_mode_instructions(mode: &ThreadMode) -> &'static str {
        match mode {
            ThreadMode::Cooking => {
                "You are a culinary assistant specializing in:\n\
                - Recipe suggestions and modifications\n\
                - Meal planning and preparation\n\
                - Ingredient substitutions\n\
                - Cooking techniques and tips\n\
                - Dietary accommodations\n\
                Track recipes discussed and modifications made."
            }
            ThreadMode::ProjectManager => {
                "You are a project management assistant specializing in:\n\
                - Task breakdown and prioritization\n\
                - Timeline and milestone planning\n\
                - Resource allocation and dependency tracking\n\
                - Risk assessment and mitigation strategies\n\
                - Progress monitoring and status reporting\n\
                Help organize work into actionable tasks with clear deliverables."
            }
            ThreadMode::General => {
                "" // General mode uses standard base instructions only
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::blocks::MemoryBlockType;
    use db::agentic_threads::{Thread, ThreadType};

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
        // Create a test thread
        let thread = Thread::create(
            &pool,
            "Test thread goal".to_string(),
            None,
            Some(ThreadType::Autonomous),
            None,
        )
        .await
        .unwrap();

        // Generate prompt without persona
        let prompt = PromptGenerator::generate_system_prompt(&pool, &thread)
            .await
            .unwrap();

        // Should contain base instructions
        assert!(prompt.contains("AI assistant"));
        assert!(prompt.contains("helpful"));

        // Should contain thread goal
        assert!(prompt.contains("Current goal: Test thread goal"));

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

        // Create a test thread
        let thread = Thread::create(
            &pool,
            "Test thread goal".to_string(),
            None,
            Some(ThreadType::Autonomous),
            None,
        )
        .await
        .unwrap();

        // Generate prompt with persona
        let prompt = PromptGenerator::generate_system_prompt(&pool, &thread)
            .await
            .unwrap();

        // Should contain base instructions
        assert!(prompt.contains("AI assistant"));

        // Should contain persona content
        assert!(prompt.contains(persona_content));
        assert!(prompt.contains("--- PERSONA MEMORY BLOCK ---"));
        assert!(prompt.contains("--- END PERSONA MEMORY BLOCK ---"));

        // Should not contain Discord instructions
        assert!(!prompt.contains("Discord"));
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_generate_system_prompt_discord_context(pool: PgPool) {
        // Create an interactive thread
        let thread = Thread::create(
            &pool,
            "Discord thread goal".to_string(),
            None,
            Some(ThreadType::Interactive),
            None,
        )
        .await
        .unwrap();

        // Generate prompt for Discord context
        let prompt = PromptGenerator::generate_system_prompt(&pool, &thread)
            .await
            .unwrap();

        // Should contain base instructions
        assert!(prompt.contains("AI assistant"));

        // Should contain thread goal
        assert!(prompt.contains("Current goal: Discord thread goal"));

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

        // Create an interactive thread
        let thread = Thread::create(
            &pool,
            "Interactive Discord goal".to_string(),
            None,
            Some(ThreadType::Interactive),
            None,
        )
        .await
        .unwrap();

        // Generate prompt for Discord context
        let prompt = PromptGenerator::generate_system_prompt(&pool, &thread)
            .await
            .unwrap();

        // Should contain all: base, goal, persona, and Discord instructions
        assert!(prompt.contains("AI assistant"));
        assert!(prompt.contains("Current goal: Interactive Discord goal"));
        assert!(prompt.contains(persona_content));
        assert!(prompt.contains("--- PERSONA MEMORY BLOCK ---"));
        assert!(prompt.contains("Discord"));
        assert!(prompt.contains("2000 characters"));
    }
}
