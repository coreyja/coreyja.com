-- Revert tool_suggestions table back to separate sample_inputs and sample_outputs columns

-- Add back the old columns
ALTER TABLE tool_suggestions ADD COLUMN sample_inputs JSONB NOT NULL DEFAULT '[]'::jsonb;
ALTER TABLE tool_suggestions ADD COLUMN sample_outputs JSONB NOT NULL DEFAULT '[]'::jsonb;

-- Migrate data back from examples to separate columns
UPDATE tool_suggestions 
SET 
    sample_inputs = COALESCE(
        (SELECT jsonb_agg(example->'input') 
         FROM jsonb_array_elements(examples) as example),
        '[]'::jsonb
    ),
    sample_outputs = COALESCE(
        (SELECT jsonb_agg(example->'output') 
         FROM jsonb_array_elements(examples) as example),
        '[]'::jsonb
    );

-- Drop the examples column
ALTER TABLE tool_suggestions DROP COLUMN examples;