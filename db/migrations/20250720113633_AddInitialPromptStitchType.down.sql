-- Revert to previous check constraint without initial_prompt
ALTER TABLE stitches
DROP CONSTRAINT IF EXISTS stitches_stitch_type_check;

ALTER TABLE stitches ADD CONSTRAINT stitches_stitch_type_check CHECK (
    stitch_type IN (
        'initial_prompt',
        'llm_call',
        'tool_call',
        'thread_result',
        'discord_message'
    )
);
