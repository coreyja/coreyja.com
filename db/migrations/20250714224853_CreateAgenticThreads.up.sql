-- Add migration script here

-- Create threads table
CREATE TABLE threads (
    thread_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_thread_id UUID REFERENCES threads(thread_id),
    branching_stitch_id UUID, -- Will add foreign key after stitches table is created
    goal TEXT NOT NULL,
    tasks JSONB DEFAULT '[]'::jsonb,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'waiting', 'completed', 'failed')),
    result JSONB,
    pending_child_results JSONB DEFAULT '[]'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create stitches table
CREATE TABLE stitches (
    stitch_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    thread_id UUID NOT NULL REFERENCES threads(thread_id),
    previous_stitch_id UUID REFERENCES stitches(stitch_id),
    stitch_type TEXT NOT NULL CHECK (stitch_type IN ('llm_call', 'tool_call', 'thread_result')),
    
    -- LLM call fields
    llm_request JSONB,
    llm_response JSONB,
    
    -- Tool call fields
    tool_name TEXT,
    tool_input JSONB,
    tool_output JSONB,
    
    -- Thread result fields (when reporting child thread completion)
    child_thread_id UUID REFERENCES threads(thread_id),
    thread_result_summary TEXT,
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Add foreign key constraint for branching_stitch_id after stitches table is created
ALTER TABLE threads ADD CONSTRAINT fk_threads_branching_stitch_id 
    FOREIGN KEY (branching_stitch_id) REFERENCES stitches(stitch_id);

-- Create indexes
CREATE INDEX idx_threads_parent_thread_id ON threads(parent_thread_id);
CREATE INDEX idx_threads_status ON threads(status);
CREATE INDEX idx_stitches_thread_id ON stitches(thread_id);
CREATE INDEX idx_stitches_previous_stitch_id ON stitches(previous_stitch_id);
CREATE UNIQUE INDEX idx_stitches_thread_previous ON stitches(thread_id, previous_stitch_id) 
    WHERE previous_stitch_id IS NOT NULL;