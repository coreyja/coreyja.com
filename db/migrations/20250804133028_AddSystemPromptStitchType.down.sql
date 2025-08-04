-- Revert the stitch_type check constraint to remove system_prompt

-- First drop the existing constraint
ALTER TABLE stitches DROP CONSTRAINT IF EXISTS stitches_stitch_type_check;

-- Add back the original constraint without system_prompt
ALTER TABLE stitches ADD CONSTRAINT stitches_stitch_type_check
CHECK (stitch_type IN ('initial_prompt', 'llm_call', 'tool_call', 'thread_result', 'discord_message'));