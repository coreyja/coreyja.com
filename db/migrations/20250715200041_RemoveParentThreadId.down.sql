-- Re-add parent_thread_id column to threads table
ALTER TABLE threads ADD COLUMN parent_thread_id UUID REFERENCES threads(thread_id);

-- Recreate the index
CREATE INDEX idx_threads_parent_thread_id ON threads(parent_thread_id);

-- Populate parent_thread_id from branching_stitch_id relationship
UPDATE threads t
SET parent_thread_id = s.thread_id
FROM stitches s
WHERE t.branching_stitch_id = s.stitch_id;