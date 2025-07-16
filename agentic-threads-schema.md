# Agentic Threads Database Schema

## Overview

The Agentic Threads system uses a threading model where each thread has a singular goal and maintains a todo-list. Threads can spawn child threads that run in parallel, with success/failure reporting back to the parent thread. The system tracks all LLM and tool interactions as "Stitches" within each thread.

## Core Tables

### Threads Table

Stores the main thread entities with their goals, status, and relationships.

```sql
CREATE TABLE threads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branching_stitch_id UUID REFERENCES stitches(id),
    goal TEXT NOT NULL,
    tasks JSONB DEFAULT '[]'::jsonb,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'waiting', 'completed', 'failed')),
    result JSONB,
    pending_child_results JSONB DEFAULT '[]'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_threads_status ON threads(status);
```

### Stitches Table

Stores individual LLM/tool interactions (stitches) within a thread, maintaining order via linked list.

```sql
CREATE TABLE stitches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    thread_id UUID NOT NULL REFERENCES threads(id),
    previous_stitch_id UUID REFERENCES stitches(id),
    stitch_type TEXT NOT NULL CHECK (stitch_type IN ('llm_call', 'tool_call', 'thread_result')),

    -- LLM call fields
    llm_request JSONB,
    llm_response JSONB,

    -- Tool call fields
    tool_name TEXT,
    tool_input JSONB,
    tool_output JSONB,

    -- Thread result fields (when reporting child thread completion)
    child_thread_id UUID REFERENCES threads(id),
    thread_result_summary TEXT,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_stitches_thread_id ON stitches(thread_id);
CREATE INDEX idx_stitches_previous_stitch_id ON stitches(previous_stitch_id);
CREATE UNIQUE INDEX idx_stitches_thread_previous ON stitches(thread_id, previous_stitch_id)
    WHERE previous_stitch_id IS NOT NULL;
```

## Key Design Decisions

### Thread Relationships

- `parent_thread_id`: Links to the parent thread that spawned this thread
- `branching_stitch_id`: References the exact stitch in the parent thread where this child was created
- Both fields allow navigation of the thread hierarchy

### Stitch Ordering

- Linked list approach using `previous_stitch_id`
- The unique index on `(thread_id, previous_stitch_id)` ensures only one stitch can follow another within a thread
- First stitch in a thread has `previous_stitch_id = NULL`

### Task Storage

- Tasks stored as JSONB in the `tasks` column
- Flexible schema allows for evolving task structure
- Example: `[{"id": "1", "description": "Parse input", "status": "done"}, {"id": "2", "description": "Generate response", "status": "pending"}]`

### Stitch Types

- `llm_call`: Stores LLM request/response
- `tool_call`: Stores tool name, input, and output
- `thread_result`: Special stitch type for child thread completion reports
  - Contains `child_thread_id` reference and `thread_result_summary`
  - Summary provides concise outcome without full execution history
  - Keeps parent thread context focused and manageable

### Concurrency Control

- Job processor can use row-level locking on thread status
- Example: `UPDATE threads SET status = 'running' WHERE id = ? AND status = 'pending' RETURNING *`
- Linear stitch structure within threads prevents race conditions

### Result Reporting

- Child thread stores result in its `result` column upon completion
- Parent thread receives a `thread_result` type stitch with the child's outcome
- If parent's last stitch is running, child results queue in `pending_child_results`
- Job processor creates thread_result stitches from pending results after current stitch completes
- Enables both direct querying of thread results and following the execution history

## Processing Flow

1. Job picks up a thread with status 'pending' or 'waiting'
2. Checks for any `pending_child_results` and creates thread_result stitches for each
3. Finds the last stitch in the thread (where no other stitch has it as `previous_stitch_id`)
4. Processes based on the last stitch type:
   - If tool call requested: Execute tool and create new stitch with results
   - If tool output exists: Send to LLM and create new stitch with response
5. If LLM requests new thread creation:
   - Create new thread with current thread as parent
   - Set `branching_stitch_id` to current stitch
   - Continue processing current thread or wait for child completion
6. When child completes:
   - Generate concise summary of child thread outcome
   - Lock parent thread row
   - If parent has running stitch: append to `pending_child_results` with summary
   - If parent is waiting: create `thread_result` stitch immediately with summary

## Sample Queries

### Find last stitch in a thread

```sql
SELECT * FROM stitches
WHERE thread_id = ?
AND id NOT IN (
    SELECT previous_stitch_id FROM stitches
    WHERE thread_id = ? AND previous_stitch_id IS NOT NULL
);
```

### Get thread execution history

```sql
WITH RECURSIVE thread_history AS (
    SELECT * FROM stitches WHERE thread_id = ? AND previous_stitch_id IS NULL
    UNION ALL
    SELECT s.* FROM stitches s
    JOIN thread_history th ON s.previous_stitch_id = th.id
)
SELECT * FROM thread_history ORDER BY created_at;
```
