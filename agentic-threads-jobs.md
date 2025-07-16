# Agentic Threads Job Architecture

## Overview

The job system uses the cja framework to process threads asynchronously. Each job execution handles one "step" of a thread, then re-enqueues itself for the next step unless the thread is complete.

## Core Jobs

### ProcessThreadStep

The main job that drives thread execution.

**Input**: 
```rust
struct ProcessThreadStepInput {
    thread_id: Uuid,
}
```

**Logic**:
1. Load thread from DB and check status
2. Lock thread (set status to 'running')
3. Process any `pending_child_results` and create thread_result Stitches
4. Find the last Stitch in the thread
5. Based on the last Stitch type:
   - If LLM response with tool call: Execute the requested tool
   - If tool output: Send to LLM for next action
   - If no Stitches yet: Send initial prompt to LLM with goal
6. Create new Stitch with the result
7. Handle any spawn_child_thread calls by creating new threads
8. If complete_thread was called: Update thread status and notify parent
9. Otherwise: Re-enqueue ProcessThreadStep for this thread_id
10. Unlock thread (set status back to 'pending' or 'waiting')

### NotifyParentThread

Handles child thread completion notifications.

**Input**:
```rust
struct NotifyParentThreadInput {
    child_thread_id: Uuid,
    result_summary: String,
    result_data: Option<serde_json::Value>,
}
```

**Logic**:
1. Lock parent thread row
2. Check if parent has a running Stitch
3. If running: Append to `pending_child_results`
4. If not: Create thread_result Stitch immediately
5. If parent was waiting for this child: Enqueue ProcessThreadStep for parent

## Cron Jobs

### CreateDailyStandup

Example cron job that creates threads on a schedule.

**Schedule**: Daily at 9 AM
**Logic**:
1. Create new Thread with goal "Generate daily standup report"
2. Set initial tasks
3. Enqueue ProcessThreadStep for the new thread

### ThreadHealthCheck

Monitors thread health and handles stuck threads.

**Schedule**: Every 5 minutes
**Logic**:
1. Find threads stuck in 'running' status for > 10 minutes
2. Reset their status to 'pending'
3. Re-enqueue ProcessThreadStep for each
4. Alert on threads that have failed multiple times

## Job Flow Diagram

```
CreateDailyStandup (cron)
    │
    ├─> Creates Thread
    │
    └─> Enqueues ProcessThreadStep
            │
            ├─> Processes one step
            │
            ├─> If spawns child:
            │   ├─> Creates child Thread
            │   └─> Enqueues ProcessThreadStep (child)
            │
            ├─> If completes:
            │   └─> Enqueues NotifyParentThread
            │
            └─> Otherwise:
                └─> Re-enqueues ProcessThreadStep (self)
```

## Error Handling

### Retry Strategy
- ProcessThreadStep: Max 3 retries with exponential backoff
- Tool execution failures: Store error in Stitch, let LLM decide next action
- LLM API failures: Retry with backoff, eventually mark thread as failed

### Thread State Management
- Always use DB transactions when updating thread state
- Use row-level locks to prevent concurrent modifications
- Timeout stuck threads via ThreadHealthCheck cron

## Observability

### Metrics to Track
- Thread creation rate
- Average thread completion time
- Tool usage frequency
- Child thread spawn rate
- Error rates by thread goal type

### Logging
- Log thread state transitions
- Log LLM prompts and responses (with sampling)
- Log tool executions and results
- Log parent-child relationships

## Future Considerations

1. **Priority Queues**: High-priority threads process first
2. **Resource Limits**: Max concurrent threads per goal type
3. **Thread Pooling**: Reuse completed threads for similar goals
4. **Webhooks**: Notify external systems on thread completion
5. **Batch Processing**: Process multiple Stitches in one job execution for efficiency