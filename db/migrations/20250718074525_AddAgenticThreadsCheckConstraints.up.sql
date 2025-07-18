-- Add check constraints to ensure data integrity for agentic threads

-- Threads table constraint: result field should only be populated when status is completed or failed
ALTER TABLE threads ADD CONSTRAINT check_result_status 
CHECK (
    (status IN ('completed', 'failed') AND result IS NOT NULL) OR 
    (status NOT IN ('completed', 'failed') AND result IS NULL)
);

-- Stitches table constraints based on stitch_type

-- LLM call fields constraint: when stitch_type = 'llm_call', only LLM fields should be populated
ALTER TABLE stitches ADD CONSTRAINT check_llm_call_fields 
CHECK (
    (stitch_type = 'llm_call' AND 
     llm_request IS NOT NULL AND 
     llm_response IS NOT NULL AND 
     tool_name IS NULL AND 
     tool_input IS NULL AND 
     tool_output IS NULL AND 
     child_thread_id IS NULL AND 
     thread_result_summary IS NULL) OR 
    (stitch_type != 'llm_call')
);

-- Tool call fields constraint: when stitch_type = 'tool_call', only tool fields should be populated
ALTER TABLE stitches ADD CONSTRAINT check_tool_call_fields 
CHECK (
    (stitch_type = 'tool_call' AND 
     tool_name IS NOT NULL AND 
     tool_input IS NOT NULL AND 
     tool_output IS NOT NULL AND 
     llm_request IS NULL AND 
     llm_response IS NULL AND 
     child_thread_id IS NULL AND 
     thread_result_summary IS NULL) OR 
    (stitch_type != 'tool_call')
);

-- Thread result fields constraint: when stitch_type = 'thread_result', only thread result fields should be populated
ALTER TABLE stitches ADD CONSTRAINT check_thread_result_fields 
CHECK (
    (stitch_type = 'thread_result' AND 
     child_thread_id IS NOT NULL AND 
     thread_result_summary IS NOT NULL AND 
     llm_request IS NULL AND 
     llm_response IS NULL AND 
     tool_name IS NULL AND 
     tool_input IS NULL AND 
     tool_output IS NULL) OR 
    (stitch_type != 'thread_result')
);