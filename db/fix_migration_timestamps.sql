-- Script to update migration version numbers in _sqlx_migrations table
-- This updates the version field to match the renamed migration files with full timestamps

-- Update CreateToolSuggestions migration
UPDATE _sqlx_migrations 
SET version = 20250718000000
WHERE version = 20250718;

-- Update UpdateToolSuggestionsToExamples migration  
UPDATE _sqlx_migrations
SET version = 20250719000000
WHERE version = 20250719;

-- Update AddPreviousStitchIdToToolSuggestions migration
UPDATE _sqlx_migrations
SET version = 20250720000000  
WHERE version = 20250720;

-- Verify the updates
SELECT version, description, checksum, execution_time 
FROM _sqlx_migrations 
WHERE version IN (20250718000000, 20250719000000, 20250720000000)
ORDER BY version;