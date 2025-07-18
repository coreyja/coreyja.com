-- Add 'aborted' as a valid status for agentic threads
ALTER TABLE threads
DROP CONSTRAINT threads_status_check;

ALTER TABLE threads ADD CONSTRAINT threads_status_check CHECK (
    status IN (
        'pending',
        'running',
        'waiting',
        'completed',
        'failed',
        'aborted'
    )
);
