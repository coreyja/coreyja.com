-- Revert the stitch_type constraint to original values
ALTER TABLE stitches DROP CONSTRAINT IF EXISTS stitches_stitch_type_check;

ALTER TABLE stitches ADD CONSTRAINT stitches_stitch_type_check 
CHECK (stitch_type IN (
    'initial_prompt',
    'system_prompt',
    'llm_call',
    'tool_call',
    'thread_result',
    'discord_message'
));