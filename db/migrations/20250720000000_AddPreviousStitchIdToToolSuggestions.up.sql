-- Add previous_stitch_id column to tool_suggestions table with foreign key to stitches
-- First add as nullable
ALTER TABLE tool_suggestions
ADD COLUMN previous_stitch_id UUID REFERENCES stitches(stitch_id);

-- Delete existing data since we can't retroactively determine the previous_stitch_id
DELETE FROM tool_suggestions;

-- Now make it NOT NULL
ALTER TABLE tool_suggestions
ALTER COLUMN previous_stitch_id SET NOT NULL;

-- Create index on previous_stitch_id for performance
CREATE INDEX idx_tool_suggestions_previous_stitch_id ON tool_suggestions(previous_stitch_id);