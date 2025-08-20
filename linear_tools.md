# Linear GraphQL Tools for AI Agent Al

## Overview
Three new tools to enable Al to interact with Linear's GraphQL API:
1. **Run GraphQL Query** - Execute queries against Linear API
2. **Search Existing Queries** - Find saved queries by name/description
3. **Save New Query** - Store queries for reuse

## Tool Specifications

### 1. Run GraphQL Query
**Purpose**: Execute GraphQL queries against Linear API and return results

**Input Parameters**:
- `query`: GraphQL query string (optional if `query_id` provided)
- `query_id`: UUID of saved query to run (optional if `query` provided)
- `variables`: JSON object of GraphQL variables (optional)

**Behavior**:
- Must provide either `query` or `query_id` (not both)
- If `query_id` provided, fetch from database
- Execute against Linear GraphQL endpoint
- Return formatted results to agent

**Response**:
- Success: JSON result from Linear
- Error: Error message with details

### 2. Search Existing Queries
**Purpose**: Find saved queries for reuse

**Input Parameters**:
- `search_term`: Text to search in name/description (optional)
- `tags`: Array of tags to filter by (optional)
- `limit`: Max results to return (default: 10)

**Behavior**:
- Search saved queries by name, description, or tags
- Return list of matching queries with metadata
- Include usage statistics (last_used, use_count)

**Response**:
- List of queries with: id, name, description, tags, created_at, last_used, use_count

### 3. Save New Query
**Purpose**: Store successful queries for future use

**Input Parameters**:
- `query`: GraphQL query string
- `name`: Human-readable name
- `description`: What the query does
- `tags`: Array of categorization tags (optional)
- `variables_schema`: JSON schema for expected variables (optional)

**Behavior**:
- Validate GraphQL syntax before saving
- Store in database with metadata
- Return saved query ID

**Response**:
- Success: Query ID and confirmation
- Error: Validation errors or save failure

## Database Schema

### Table: `linear_saved_queries`
```sql
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
```

### Table: `linear_query_usage`
```sql
CREATE TABLE linear_query_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    query_id UUID REFERENCES linear_saved_queries(id) ON DELETE CASCADE,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    variables JSONB,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    response_size_bytes INTEGER,
    execution_time_ms INTEGER
);

CREATE INDEX idx_query_usage_query_id ON linear_query_usage(query_id);
CREATE INDEX idx_query_usage_executed_at ON linear_query_usage(executed_at);
```

## Implementation Plan

### Phase 1: Database Setup
1. Create migration for `linear_saved_queries` table
2. Create migration for `linear_query_usage` table
3. Add SQLx models in `/db/src/models/`

### Phase 2: Tool Implementation
1. Create tool handlers in `/server/src/agent/tools/`
2. Leverage existing Linear GraphQL client and authentication
3. Use stored Linear token from database for API calls
4. Implement each tool with proper error handling
5. Add validation for GraphQL queries
