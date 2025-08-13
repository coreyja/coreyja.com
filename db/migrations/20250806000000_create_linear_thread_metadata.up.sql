-- Create Linear thread metadata table to store Linear-specific information for interactive threads
CREATE TABLE linear_thread_metadata (
    thread_id UUID PRIMARY KEY REFERENCES threads(thread_id) ON DELETE CASCADE,
    session_id VARCHAR(255) NOT NULL UNIQUE,
    workspace_id VARCHAR(255) NOT NULL,
    issue_id VARCHAR(255),
    issue_title TEXT,
    project_id VARCHAR(255),
    team_id VARCHAR(255),
    created_by_user_id VARCHAR(255) NOT NULL,
    session_status TEXT NOT NULL DEFAULT 'pending',
    last_activity_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    CONSTRAINT check_session_status CHECK (
        session_status IN ('pending', 'active', 'error', 'awaitingInput', 'complete')
    )
);

-- Create indexes for fast lookups
CREATE INDEX idx_linear_thread_metadata_session_id ON linear_thread_metadata(session_id);
CREATE INDEX idx_linear_thread_metadata_issue_id ON linear_thread_metadata(issue_id);
CREATE INDEX idx_linear_thread_metadata_workspace_id ON linear_thread_metadata(workspace_id);
CREATE INDEX idx_linear_thread_metadata_team_id ON linear_thread_metadata(team_id);
CREATE INDEX idx_linear_thread_metadata_last_activity_at ON linear_thread_metadata(last_activity_at);

-- Add trigger to update updated_at on row updates
CREATE OR REPLACE FUNCTION update_linear_thread_metadata_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_linear_thread_metadata_updated_at
    BEFORE UPDATE ON linear_thread_metadata
    FOR EACH ROW
    EXECUTE FUNCTION update_linear_thread_metadata_updated_at();