-- Drop migration script here

-- Drop indexes
DROP INDEX IF EXISTS idx_stitches_thread_previous;
DROP INDEX IF EXISTS idx_stitches_previous_stitch_id;
DROP INDEX IF EXISTS idx_stitches_thread_id;
DROP INDEX IF EXISTS idx_threads_status;
DROP INDEX IF EXISTS idx_threads_parent_thread_id;

-- Drop foreign key constraint
ALTER TABLE threads DROP CONSTRAINT IF EXISTS fk_threads_branching_stitch_id;

-- Drop tables
DROP TABLE IF EXISTS stitches;
DROP TABLE IF EXISTS threads;