-- Add system_prompt to the stitch_type check constraint
-- This allows us to separate system prompts from initial user messages

-- First drop the existing constraint
ALTER TABLE stitches DROP CONSTRAINT IF EXISTS stitches_stitch_type_check;

-- Add the new constraint with system_prompt included
ALTER TABLE stitches ADD CONSTRAINT stitches_stitch_type_check
CHECK (stitch_type IN ('initial_prompt', 'llm_call', 'tool_call', 'thread_result', 'discord_message', 'system_prompt'));