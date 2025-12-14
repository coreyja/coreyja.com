-- Revert agent_response back to agent_thought

-- First, drop the constraint so we can update the records
ALTER TABLE stitches DROP CONSTRAINT IF EXISTS stitches_stitch_type_check;

-- Then update existing records back
UPDATE stitches
SET stitch_type = 'agent_thought'
WHERE stitch_type = 'agent_response';

-- Finally, add the old constraint back
ALTER TABLE stitches ADD CONSTRAINT stitches_stitch_type_check
CHECK (stitch_type IN (
    'initial_prompt',
    'system_prompt',
    'llm_call',
    'tool_call',
    'thread_result',
    'discord_message',
    'agent_thought',
    'clarification_request',
    'error'
));
