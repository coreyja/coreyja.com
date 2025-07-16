-- Remove parent_thread_id from threads table since threads will be branched through stitches only
-- The parent relationship can be derived from branching_stitch_id

-- Drop the index first
DROP INDEX IF EXISTS idx_threads_parent_thread_id;

-- Drop the column
ALTER TABLE threads DROP COLUMN parent_thread_id;