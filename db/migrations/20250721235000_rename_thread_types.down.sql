-- Revert thread_type values
UPDATE threads 
SET thread_type = 'goal_oriented' 
WHERE thread_type = 'autonomous';

-- Drop the new check constraint
ALTER TABLE threads 
DROP CONSTRAINT IF EXISTS threads_thread_type_check;

-- Add back the old check constraint
ALTER TABLE threads 
ADD CONSTRAINT threads_thread_type_check 
CHECK (thread_type IN ('goal_oriented', 'interactive'));

-- Revert the default value
ALTER TABLE threads 
ALTER COLUMN thread_type SET DEFAULT 'goal_oriented';