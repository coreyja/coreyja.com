use color_eyre::Result;
use db::agentic_threads::{Thread, ThreadType};
use sqlx::PgPool;
use std::fmt::Write;

use super::blocks::MemoryBlock;

#[derive(Debug)]
pub struct PromptGenerator;

impl PromptGenerator {
    pub fn new() -> Self {
        Self
    }

    pub async fn generate_system_prompt(
        pool: &PgPool,
        thread: &Thread,
        person_identifier: Option<String>,
        persona: Option<String>,
    ) -> Result<String> {
        // Base instructions (always included)
        let mut system_content = Self::base_instructions().to_string();

        // Add thread goal
        write!(system_content, "\nCurrent goal: {}\n", thread.goal)?;

        // Add persona if available - use provided identifier or default to "default"
        let persona_lookup_id = persona.unwrap_or_else(|| "default".to_string());
        let persona_block = MemoryBlock::find_by_type_and_identifier(
            pool,
            "persona".to_string(),
            persona_lookup_id.clone(),
        )
        .await?;

        if let Some(block) = persona_block {
            system_content.push_str("\n--- PERSONA MEMORY BLOCK ---\n");
            system_content.push_str(&block.content);
            system_content.push_str("\n--- END PERSONA MEMORY BLOCK ---\n");
        }

        // Add person memory if identifier provided
        if let Some(identifier) = person_identifier {
            if let Some(person_block) =
                MemoryBlock::find_by_type_and_identifier(pool, "person".to_string(), identifier)
                    .await?
            {
                system_content.push_str("\n--- PERSON MEMORY BLOCK ---\n");
                system_content.push_str(&person_block.content);
                system_content.push_str("\n--- END PERSON MEMORY BLOCK ---\n");
            }
            // Fail gracefully if person memory doesn't exist - just skip injection
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
}

#[cfg(test)]
mod tests {
    use super::*;
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
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();

        // Generate prompt without persona (pass None for person_identifier)
        let prompt = PromptGenerator::generate_system_prompt(&pool, &thread, None, None)
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
        MemoryBlock::create(
            &pool,
            "persona".to_string(),
            "default".to_string(),
            persona_content.to_string(),
        )
        .await
        .unwrap();

        // Create a test thread
        let thread = Thread::create(
            &pool,
            "Test thread goal".to_string(),
            None,
            Some(ThreadType::Autonomous),
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();

        // Generate prompt with persona (pass None for person_identifier)
        let prompt = PromptGenerator::generate_system_prompt(&pool, &thread, None, None)
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
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();

        // Generate prompt for Discord context (pass None for person_identifier)
        let prompt = PromptGenerator::generate_system_prompt(&pool, &thread, None, None)
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
        MemoryBlock::create(
            &pool,
            "persona".to_string(),
            "default".to_string(),
            persona_content.to_string(),
        )
        .await
        .unwrap();

        // Create an interactive thread
        let thread = Thread::create(
            &pool,
            "Interactive Discord goal".to_string(),
            None,
            Some(ThreadType::Interactive),
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();

        // Generate prompt for Discord context (pass None for person_identifier)
        let prompt = PromptGenerator::generate_system_prompt(&pool, &thread, None, None)
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

    // New tests for person memory injection (Task 4.1)

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_generate_system_prompt_with_person_memory(pool: PgPool) {
        // Create persona
        MemoryBlock::create(
            &pool,
            "persona".to_string(),
            "default".to_string(),
            "I am a friendly AI assistant.".to_string(),
        )
        .await
        .unwrap();

        // Create person memory
        let person_content =
            "Corey is a software engineer who loves Rust and functional programming.";
        MemoryBlock::create(
            &pool,
            "person".to_string(),
            "corey#1234".to_string(),
            person_content.to_string(),
        )
        .await
        .unwrap();

        // Create thread
        let thread = Thread::create(
            &pool,
            "Help Corey with code".to_string(),
            None,
            Some(ThreadType::Interactive),
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();

        // Generate prompt with person identifier
        let prompt = PromptGenerator::generate_system_prompt(
            &pool,
            &thread,
            Some("corey#1234".to_string()),
            None,
        )
        .await
        .unwrap();

        // Should contain person memory block
        assert!(prompt.contains("--- PERSON MEMORY BLOCK ---"));
        assert!(prompt.contains(person_content));
        assert!(prompt.contains("--- END PERSON MEMORY BLOCK ---"));

        // Should also contain persona
        assert!(prompt.contains("--- PERSONA MEMORY BLOCK ---"));

        // Person memory should come after persona
        let persona_pos = prompt.find("--- PERSONA MEMORY BLOCK ---").unwrap();
        let person_pos = prompt.find("--- PERSON MEMORY BLOCK ---").unwrap();
        assert!(person_pos > persona_pos);
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_generate_system_prompt_person_memory_not_found(pool: PgPool) {
        // Create thread
        let thread = Thread::create(
            &pool,
            "Test goal".to_string(),
            None,
            Some(ThreadType::Interactive),
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();

        // Generate prompt with non-existent person identifier - should not error
        let prompt = PromptGenerator::generate_system_prompt(
            &pool,
            &thread,
            Some("nonexistent#0000".to_string()),
            None,
        )
        .await
        .unwrap();

        // Should NOT contain person memory block
        assert!(!prompt.contains("--- PERSON MEMORY BLOCK ---"));

        // Should still contain base instructions
        assert!(prompt.contains("AI assistant"));
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_generate_system_prompt_person_memory_without_persona(pool: PgPool) {
        // Create person memory only (no persona)
        let person_content = "Jane is a product manager who enjoys design thinking.";
        MemoryBlock::create(
            &pool,
            "person".to_string(),
            "jane#5678".to_string(),
            person_content.to_string(),
        )
        .await
        .unwrap();

        // Create thread
        let thread = Thread::create(
            &pool,
            "Discuss product strategy".to_string(),
            None,
            Some(ThreadType::Autonomous),
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();

        // Generate prompt with person identifier but no persona
        let prompt = PromptGenerator::generate_system_prompt(
            &pool,
            &thread,
            Some("jane#5678".to_string()),
            None,
        )
        .await
        .unwrap();

        // Should contain person memory
        assert!(prompt.contains("--- PERSON MEMORY BLOCK ---"));
        assert!(prompt.contains(person_content));

        // Should NOT contain persona memory
        assert!(!prompt.contains("--- PERSONA MEMORY BLOCK ---"));
    }

    #[sqlx::test(migrations = "../db/migrations")]
    async fn test_generate_system_prompt_person_memory_order(pool: PgPool) {
        // Create both persona and person memory
        MemoryBlock::create(
            &pool,
            "persona".to_string(),
            "default".to_string(),
            "I am helpful.".to_string(),
        )
        .await
        .unwrap();

        MemoryBlock::create(
            &pool,
            "person".to_string(),
            "user#0001".to_string(),
            "This user prefers concise answers.".to_string(),
        )
        .await
        .unwrap();

        let thread = Thread::create(
            &pool,
            "Answer questions".to_string(),
            None,
            Some(ThreadType::Interactive),
            crate::agent_config::DEFAULT_AGENT_ID.to_string(),
        )
        .await
        .unwrap();

        // Generate prompt
        let prompt = PromptGenerator::generate_system_prompt(
            &pool,
            &thread,
            Some("user#0001".to_string()),
            None,
        )
        .await
        .unwrap();

        // Verify order: base instructions, goal, persona, person, discord instructions
        let base_pos = prompt.find("AI assistant").unwrap();
        let goal_pos = prompt.find("Current goal:").unwrap();
        let persona_pos = prompt.find("--- PERSONA MEMORY BLOCK ---").unwrap();
        let person_pos = prompt.find("--- PERSON MEMORY BLOCK ---").unwrap();
        let discord_pos = prompt.find("Discord").unwrap();

        assert!(base_pos < goal_pos);
        assert!(goal_pos < persona_pos);
        assert!(persona_pos < person_pos);
        assert!(person_pos < discord_pos);
    }
}
