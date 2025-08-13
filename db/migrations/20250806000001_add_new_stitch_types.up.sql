-- Drop the existing check constraint for stitch_type
ALTER TABLE stitches DROP CONSTRAINT IF EXISTS stitches_stitch_type_check;

-- Add the new constraint with additional stitch types for Linear integration
ALTER TABLE stitches ADD CONSTRAINT stitches_stitch_type_check 
CHECK (stitch_type IN (
    'initial_prompt',
    'system_prompt',
    'llm_call',
    'tool_call',
    'thread_result',
    'discord_message',
    'agent_thought',          -- NEW: Internal agent reasoning
    'clarification_request',  -- NEW: Requesting user clarification  
    'error'                   -- NEW: Error states
));