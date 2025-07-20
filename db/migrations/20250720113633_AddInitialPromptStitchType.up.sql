-- Add check constraint to include the new initial_prompt stitch type
ALTER TABLE stitches DROP CONSTRAINT IF EXISTS stitches_stitch_type_check;
ALTER TABLE stitches ADD CONSTRAINT stitches_stitch_type_check 
    CHECK (stitch_type IN ('llm_call', 'tool_call', 'thread_result', 'initial_prompt'));