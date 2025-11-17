-- Rename agent_thought stitch type to agent_response for clarity
-- This makes it clear that these stitches represent responses sent to users,
-- not to be confused with Anthropic's thinking blocks feature

-- First, drop the constraint so we can update the records
ALTER TABLE stitches DROP CONSTRAINT IF EXISTS stitches_stitch_type_check;

-- Then update existing records
UPDATE stitches
SET stitch_type = 'agent_response'
WHERE stitch_type = 'agent_thought';

-- Finally, add the new constraint with the updated value
ALTER TABLE stitches ADD CONSTRAINT stitches_stitch_type_check
CHECK (stitch_type IN (
    'initial_prompt',
    'system_prompt',
    'llm_call',
    'tool_call',
    'thread_result',
    'discord_message',
    'agent_response',         -- Renamed from agent_thought
    'clarification_request',
    'error'
));
