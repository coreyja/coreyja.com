-- Create table for saved Linear GraphQL queries
CREATE TABLE linear_saved_queries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    query TEXT NOT NULL,
    variables_schema JSONB,
    tags TEXT[],
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by VARCHAR(255), -- Could be 'al' or user identifier
    UNIQUE(name)
);

CREATE INDEX idx_linear_queries_tags ON linear_saved_queries USING GIN(tags);
CREATE INDEX idx_linear_queries_name ON linear_saved_queries(name);

-- Create table for tracking query usage
CREATE TABLE linear_query_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    query_id UUID NOT NULL REFERENCES linear_saved_queries(id) ON DELETE CASCADE,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    variables JSONB,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    response_size_bytes INTEGER,
    execution_time_ms INTEGER
);

CREATE INDEX idx_query_usage_query_id ON linear_query_usage(query_id);
CREATE INDEX idx_query_usage_executed_at ON linear_query_usage(executed_at);
