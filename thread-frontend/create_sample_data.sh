#!/bin/bash

# Database connection URL
DATABASE_URL="${DATABASE_URL:-postgresql://user:password@localhost:5432/database}"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Creating sample data for Agentic Threads UI...${NC}"

# Create the SQL commands
psql "$DATABASE_URL" << EOF

-- Clean up existing data by truncating tables
TRUNCATE TABLE stitches, threads RESTART IDENTITY CASCADE;

-- Create main thread 1: Code Review Assistant
INSERT INTO threads (thread_id, branching_stitch_id, goal, tasks, status, result, pending_child_results, created_at, updated_at)
VALUES 
  ('a1b2c3d4-e5f6-7890-abcd-ef1234567801', NULL, 'Sample: Review and improve code quality', 
   '[{"id": "task-1", "description": "Analyze code structure", "status": "completed"},
     {"id": "task-2", "description": "Check for security issues", "status": "completed"},
     {"id": "task-3", "description": "Suggest improvements", "status": "in_progress"}]'::jsonb,
   'running', NULL, '[]'::jsonb, NOW() - INTERVAL '2 hours', NOW());

-- Create main thread 3: Data Processing Pipeline
INSERT INTO threads (thread_id, branching_stitch_id, goal, tasks, status, result, pending_child_results, created_at, updated_at)
VALUES 
  ('a3b2c3d4-e5f6-7890-abcd-ef1234567803', NULL, 'Sample: Process and analyze user data', 
   '[{"id": "task-1", "description": "Load data from database", "status": "completed"},
     {"id": "task-2", "description": "Transform data", "status": "completed"},
     {"id": "task-3", "description": "Generate reports", "status": "running"}]'::jsonb,
   'running', NULL, 
   '[{"thread_id": "a4b2c3d4-e5f6-7890-abcd-ef1234567804", "result": null}, {"thread_id": "a5b2c3d4-e5f6-7890-abcd-ef1234567805", "result": null}]'::jsonb, 
   NOW() - INTERVAL '1 hour', NOW());

-- Create a standalone failed thread
INSERT INTO threads (thread_id, branching_stitch_id, goal, tasks, status, result, pending_child_results, created_at, updated_at)
VALUES 
  ('a7b2c3d4-e5f6-7890-abcd-ef1234567807', NULL, 'Sample: Failed task - API integration', 
   '[{"id": "task-1", "description": "Connect to external API", "status": "completed"},
     {"id": "task-2", "description": "Fetch data", "status": "in_progress"}]'::jsonb,
   'failed', 
   '{"success": false, "error": "API rate limit exceeded"}'::jsonb, 
   '[]'::jsonb, NOW() - INTERVAL '3 hours', NOW() - INTERVAL '2 hours 30 minutes');

-- Add stitches for thread 1
INSERT INTO stitches (stitch_id, thread_id, previous_stitch_id, stitch_type, llm_request, llm_response, tool_name, tool_input, tool_output, child_thread_id, thread_result_summary, created_at)
VALUES
  ('b1b2c3d4-e5f6-7890-abcd-ef1234567801', 'a1b2c3d4-e5f6-7890-abcd-ef1234567801', NULL, 'llm_call', 
   '{"model": "gpt-4", "messages": [{"role": "user", "content": "Analyze this code structure"}]}'::jsonb,
   '{"choices": [{"message": {"content": "I will analyze the code structure..."}}]}'::jsonb,
   NULL, NULL, NULL, NULL, NULL, NOW() - INTERVAL '2 hours'),
  
  ('b2b2c3d4-e5f6-7890-abcd-ef1234567802', 'a1b2c3d4-e5f6-7890-abcd-ef1234567801', 'b1b2c3d4-e5f6-7890-abcd-ef1234567801', 'tool_call', 
   NULL, NULL, 'code_analyzer', 
   '{"file": "main.py", "analysis_type": "structure"}'::jsonb,
   '{"complexity": 7, "lines": 234, "functions": 12}'::jsonb,
   NULL, NULL, NOW() - INTERVAL '1 hour 50 minutes');

-- Add stitch that spawns thread 2
INSERT INTO stitches (stitch_id, thread_id, previous_stitch_id, stitch_type, llm_request, llm_response, tool_name, tool_input, tool_output, child_thread_id, thread_result_summary, created_at)
VALUES
  ('b3b2c3d4-e5f6-7890-abcd-ef1234567803', 'a1b2c3d4-e5f6-7890-abcd-ef1234567801', 'b2b2c3d4-e5f6-7890-abcd-ef1234567802', 'thread_result', 
   NULL, NULL, NULL, NULL, NULL, 'a2b2c3d4-e5f6-7890-abcd-ef1234567802', 
   'Security scan completed: 2 medium issues found', NOW() - INTERVAL '1 hour 40 minutes');

-- Create child thread 2: Security Scanner (spawned from stitch b3b2c3d4-e5f6-7890-abcd-ef1234567803)
INSERT INTO threads (thread_id, branching_stitch_id, goal, tasks, status, result, pending_child_results, created_at, updated_at)
VALUES 
  ('a2b2c3d4-e5f6-7890-abcd-ef1234567802', 'b3b2c3d4-e5f6-7890-abcd-ef1234567803', 'Sample: Perform security analysis', 
   '[{"id": "task-1", "description": "Scan for vulnerabilities", "status": "completed"},
     {"id": "task-2", "description": "Check dependencies", "status": "completed"}]'::jsonb,
   'completed', 
   '{"success": true, "data": {"vulnerabilities": 2, "severity": "medium"}}'::jsonb, 
   '[]'::jsonb, NOW() - INTERVAL '1 hour 40 minutes', NOW() - INTERVAL '30 minutes');

-- Add stitches for thread 2
INSERT INTO stitches (stitch_id, thread_id, previous_stitch_id, stitch_type, llm_request, llm_response, tool_name, tool_input, tool_output, child_thread_id, thread_result_summary, created_at)
VALUES
  ('b4b2c3d4-e5f6-7890-abcd-ef1234567804', 'a2b2c3d4-e5f6-7890-abcd-ef1234567802', NULL, 'tool_call', 
   NULL, NULL, 'security_scanner', 
   '{"scan_type": "full", "include_deps": true}'::jsonb,
   '{"vulnerabilities": [{"type": "SQL_INJECTION", "severity": "medium"}, {"type": "XSS", "severity": "medium"}]}'::jsonb,
   NULL, NULL, NOW() - INTERVAL '1 hour 30 minutes'),
  
  ('b5b2c3d4-e5f6-7890-abcd-ef1234567805', 'a2b2c3d4-e5f6-7890-abcd-ef1234567802', 'b4b2c3d4-e5f6-7890-abcd-ef1234567804', 'llm_call', 
   '{"model": "gpt-4", "messages": [{"role": "user", "content": "Summarize security findings"}]}'::jsonb,
   '{"choices": [{"message": {"content": "Found 2 medium severity issues..."}}]}'::jsonb,
   NULL, NULL, NULL, NULL, NULL, NOW() - INTERVAL '1 hour 20 minutes');

-- Add stitches for thread 3
INSERT INTO stitches (stitch_id, thread_id, previous_stitch_id, stitch_type, llm_request, llm_response, tool_name, tool_input, tool_output, child_thread_id, thread_result_summary, created_at)
VALUES
  ('b6b2c3d4-e5f6-7890-abcd-ef1234567806', 'a3b2c3d4-e5f6-7890-abcd-ef1234567803', NULL, 'tool_call', 
   NULL, NULL, 'database_query', 
   '{"query": "SELECT * FROM users WHERE created_at > NOW() - INTERVAL ''7 days''"}'::jsonb,
   '{"row_count": 1523, "execution_time": "45ms"}'::jsonb,
   NULL, NULL, NOW() - INTERVAL '50 minutes');

-- Add stitch that spawns thread 4
INSERT INTO stitches (stitch_id, thread_id, previous_stitch_id, stitch_type, llm_request, llm_response, tool_name, tool_input, tool_output, child_thread_id, thread_result_summary, created_at)
VALUES
  ('b7b2c3d4-e5f6-7890-abcd-ef1234567807', 'a3b2c3d4-e5f6-7890-abcd-ef1234567803', 'b6b2c3d4-e5f6-7890-abcd-ef1234567806', 'thread_result', 
   NULL, NULL, NULL, NULL, NULL, 'a4b2c3d4-e5f6-7890-abcd-ef1234567804', 
   'Data transformation completed', NOW() - INTERVAL '40 minutes');

-- Create child thread 4: Data Transformer
INSERT INTO threads (thread_id, branching_stitch_id, goal, tasks, status, result, pending_child_results, created_at, updated_at)
VALUES 
  ('a4b2c3d4-e5f6-7890-abcd-ef1234567804', 'b7b2c3d4-e5f6-7890-abcd-ef1234567807', 'Sample: Transform raw data', 
   '[{"id": "task-1", "description": "Clean data", "status": "completed"},
     {"id": "task-2", "description": "Normalize values", "status": "completed"}]'::jsonb,
   'completed', 
   '{"success": true, "data": {"records_processed": 1523}}'::jsonb, 
   '[]'::jsonb, NOW() - INTERVAL '40 minutes', NOW() - INTERVAL '35 minutes');

-- Add stitch that spawns thread 5
INSERT INTO stitches (stitch_id, thread_id, previous_stitch_id, stitch_type, llm_request, llm_response, tool_name, tool_input, tool_output, child_thread_id, thread_result_summary, created_at)
VALUES
  ('b8b2c3d4-e5f6-7890-abcd-ef1234567808', 'a3b2c3d4-e5f6-7890-abcd-ef1234567803', 'b7b2c3d4-e5f6-7890-abcd-ef1234567807', 'thread_result', 
   NULL, NULL, NULL, NULL, NULL, 'a5b2c3d4-e5f6-7890-abcd-ef1234567805', 
   'Report generation in progress', NOW() - INTERVAL '30 minutes');

-- Create child thread 5: Report Generator
INSERT INTO threads (thread_id, branching_stitch_id, goal, tasks, status, result, pending_child_results, created_at, updated_at)
VALUES 
  ('a5b2c3d4-e5f6-7890-abcd-ef1234567805', 'b8b2c3d4-e5f6-7890-abcd-ef1234567808', 'Sample: Generate analytics reports', 
   '[{"id": "task-1", "description": "Create visualizations", "status": "completed"},
     {"id": "task-2", "description": "Generate PDF", "status": "in_progress"}]'::jsonb,
   'running', NULL, '[]'::jsonb, NOW() - INTERVAL '30 minutes', NOW());

-- Add stitches for thread 5
INSERT INTO stitches (stitch_id, thread_id, previous_stitch_id, stitch_type, llm_request, llm_response, tool_name, tool_input, tool_output, child_thread_id, thread_result_summary, created_at)
VALUES
  ('b9b2c3d4-e5f6-7890-abcd-ef1234567809', 'a5b2c3d4-e5f6-7890-abcd-ef1234567805', NULL, 'tool_call', 
   NULL, NULL, 'chart_generator', 
   '{"chart_type": "bar", "data": {"labels": ["Mon", "Tue", "Wed"], "values": [100, 150, 120]}}'::jsonb,
   '{"chart_url": "/charts/weekly_activity.png"}'::jsonb,
   NULL, NULL, NOW() - INTERVAL '25 minutes');

-- Add stitch that spawns thread 6
INSERT INTO stitches (stitch_id, thread_id, previous_stitch_id, stitch_type, llm_request, llm_response, tool_name, tool_input, tool_output, child_thread_id, thread_result_summary, created_at)
VALUES
  ('bab2c3d4-e5f6-7890-abcd-ef1234567810', 'a5b2c3d4-e5f6-7890-abcd-ef1234567805', 'b9b2c3d4-e5f6-7890-abcd-ef1234567809', 'thread_result', 
   NULL, NULL, NULL, NULL, NULL, 'a6b2c3d4-e5f6-7890-abcd-ef1234567806', 
   'Export to multiple formats', NOW() - INTERVAL '20 minutes');

-- Create grandchild thread 6: Multi-format Exporter
INSERT INTO threads (thread_id, branching_stitch_id, goal, tasks, status, result, pending_child_results, created_at, updated_at)
VALUES 
  ('a6b2c3d4-e5f6-7890-abcd-ef1234567806', 'bab2c3d4-e5f6-7890-abcd-ef1234567810', 'Sample: Export report to PDF, Excel, and email', 
   '[{"id": "task-1", "description": "Generate PDF", "status": "completed"},
     {"id": "task-2", "description": "Create Excel file", "status": "in_progress"},
     {"id": "task-3", "description": "Send email", "status": "pending"}]'::jsonb,
   'running', NULL, '[]'::jsonb, NOW() - INTERVAL '20 minutes', NOW());

-- Add error stitch for failed thread
INSERT INTO stitches (stitch_id, thread_id, previous_stitch_id, stitch_type, llm_request, llm_response, tool_name, tool_input, tool_output, child_thread_id, thread_result_summary, created_at)
VALUES
  ('bbb2c3d4-e5f6-7890-abcd-ef1234567811', 'a7b2c3d4-e5f6-7890-abcd-ef1234567807', NULL, 'tool_call', 
   NULL, NULL, 'api_client', 
   '{"endpoint": "/v1/data", "method": "GET"}'::jsonb,
   '{"error": "Rate limit exceeded", "retry_after": 3600}'::jsonb,
   NULL, NULL, NOW() - INTERVAL '2 hours 35 minutes');

-- Add stitch to thread 4 to show it has no child threads
INSERT INTO stitches (stitch_id, thread_id, previous_stitch_id, stitch_type, llm_request, llm_response, tool_name, tool_input, tool_output, child_thread_id, thread_result_summary, created_at)
VALUES
  ('bcb2c3d4-e5f6-7890-abcd-ef1234567812', 'a4b2c3d4-e5f6-7890-abcd-ef1234567804', NULL, 'tool_call', 
   NULL, NULL, 'data_cleaner', 
   '{"rows_processed": 1523, "invalid_rows": 12}'::jsonb,
   '{"cleaned_rows": 1511, "execution_time": "120ms"}'::jsonb,
   NULL, NULL, NOW() - INTERVAL '38 minutes');

-- Add stitch to thread 6 to show it has no child threads
INSERT INTO stitches (stitch_id, thread_id, previous_stitch_id, stitch_type, llm_request, llm_response, tool_name, tool_input, tool_output, child_thread_id, thread_result_summary, created_at)
VALUES
  ('bdb2c3d4-e5f6-7890-abcd-ef1234567813', 'a6b2c3d4-e5f6-7890-abcd-ef1234567806', NULL, 'tool_call', 
   NULL, NULL, 'pdf_generator', 
   '{"template": "report", "data": {"title": "Weekly Analytics"}}'::jsonb,
   '{"file_path": "/exports/report_2024.pdf", "size": "2.3MB"}'::jsonb,
   NULL, NULL, NOW() - INTERVAL '15 minutes');

EOF

echo -e "${GREEN}Sample data created successfully!${NC}"
echo -e "${BLUE}Summary of created data:${NC}"
echo "- 7 threads (including nested child threads)"
echo "- 13 stitches showing different types (llm_call, tool_call, thread_result)"
echo "- Various thread statuses: running, completed, failed"
echo "- Nested thread hierarchy up to 3 levels deep"
echo "- All child threads properly linked to their spawning stitches"
echo ""
echo -e "${BLUE}To use this script:${NC}"
echo "1. Set DATABASE_URL environment variable"
echo "   Example: export DATABASE_URL='postgresql://user:password@localhost:5432/database'"
echo "2. Run: ./create_sample_data.sh"