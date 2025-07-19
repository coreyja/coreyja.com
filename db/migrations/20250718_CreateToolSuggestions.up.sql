-- Create tool_suggestions table for agents to suggest new tools
CREATE TABLE tool_suggestions (
    suggestion_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    sample_inputs JSONB NOT NULL DEFAULT '[]'::jsonb,
    sample_outputs JSONB NOT NULL DEFAULT '[]'::jsonb,
    status TEXT NOT NULL DEFAULT 'pending',
    linear_ticket_id TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- Check constraints
    CONSTRAINT check_status CHECK (status IN ('pending', 'dismissed', 'skipped')),
    CONSTRAINT check_linear_ticket_id CHECK (
        (status = 'dismissed' AND linear_ticket_id IS NOT NULL) OR
        (status != 'dismissed' AND linear_ticket_id IS NULL)
    ),
    CONSTRAINT check_name_not_empty CHECK (length(trim(name)) > 0),
    CONSTRAINT check_description_not_empty CHECK (length(trim(description)) > 0)
);

-- Create indexes
CREATE INDEX idx_tool_suggestions_status ON tool_suggestions(status);
CREATE INDEX idx_tool_suggestions_created_at ON tool_suggestions(created_at DESC);

-- Create trigger to update updated_at
CREATE OR REPLACE FUNCTION update_tool_suggestions_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_tool_suggestions_updated_at
    BEFORE UPDATE ON tool_suggestions
    FOR EACH ROW
    EXECUTE FUNCTION update_tool_suggestions_updated_at();