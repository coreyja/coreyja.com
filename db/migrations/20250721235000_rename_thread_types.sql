-- Update existing thread_type values
UPDATE threads 
SET thread_type = 'autonomous' 
WHERE thread_type = 'goal_oriented';

-- Drop the old check constraint
ALTER TABLE threads 
DROP CONSTRAINT IF EXISTS threads_thread_type_check;

-- Add the new check constraint with renamed values
ALTER TABLE threads 
ADD CONSTRAINT threads_thread_type_check 
CHECK (thread_type IN ('autonomous', 'interactive'));

-- Update the default value
ALTER TABLE threads 
ALTER COLUMN thread_type SET DEFAULT 'autonomous';