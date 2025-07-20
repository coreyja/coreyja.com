-- Script to check current migration versions before updating
-- Run this first to verify which migrations need to be updated

-- Check for migrations with date-only versions (problematic ones)
SELECT version, description, checksum, execution_time 
FROM _sqlx_migrations 
WHERE version IN (20250718, 20250719, 20250720)
ORDER BY version;

-- Also check if any migrations already have the full timestamp versions
-- (in case some environments were already updated)
SELECT version, description, checksum, execution_time 
FROM _sqlx_migrations 
WHERE version IN (20250718000000, 20250719000000, 20250720000000)
ORDER BY version;

-- Show all recent migrations for context
SELECT version, description, checksum, execution_time 
FROM _sqlx_migrations 
WHERE version >= 20250714
ORDER BY version;