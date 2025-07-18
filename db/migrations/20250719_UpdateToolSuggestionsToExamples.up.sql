-- Update tool_suggestions table to use a single examples column instead of separate inputs/outputs

-- Add the new examples column
ALTER TABLE tool_suggestions ADD COLUMN examples JSONB NOT NULL DEFAULT '[]'::jsonb;

-- Migrate existing data by combining inputs and outputs into examples array
UPDATE tool_suggestions 
SET examples = (
    SELECT jsonb_agg(
        jsonb_build_object(
            'input', input_val,
            'output', COALESCE(output_val, 'null'::jsonb)
        )
    )
    FROM (
        SELECT 
            jsonb_array_elements(sample_inputs) as input_val,
            jsonb_array_elements(sample_outputs) as output_val
        FROM tool_suggestions ts
        WHERE ts.suggestion_id = tool_suggestions.suggestion_id
    ) as combined
);

-- Drop the old columns
ALTER TABLE tool_suggestions DROP COLUMN sample_inputs;
ALTER TABLE tool_suggestions DROP COLUMN sample_outputs;