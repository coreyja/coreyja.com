# Agentic Threads Tool Functions

## Overview

All agent actions are implemented as function calls in the OpenAI-like API format. These tools allow agents to understand their thread context, manage tasks, spawn child threads, and interact with the system.

## Core Thread Management Tools

### get_current_thread_info
Returns information about the current thread including goal, status, and task list.

```json
{
  "name": "get_current_thread_info",
  "description": "Get information about the current thread",
  "parameters": {}
}
```

### update_tasks
Update the task list for the current thread.

```json
{
  "name": "update_tasks",
  "description": "Update the todo list for the current thread",
  "parameters": {
    "type": "object",
    "properties": {
      "tasks": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "id": { "type": "string" },
            "description": { "type": "string" },
            "status": { "type": "string", "enum": ["pending", "in_progress", "completed", "blocked"] }
          }
        }
      }
    },
    "required": ["tasks"]
  }
}
```

### spawn_child_thread
Create a new child thread with a specific goal.

```json
{
  "name": "spawn_child_thread",
  "description": "Create a child thread to handle a specific sub-task",
  "parameters": {
    "type": "object",
    "properties": {
      "goal": {
        "type": "string",
        "description": "The specific goal for the child thread"
      },
      "initial_tasks": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "description": { "type": "string" }
          }
        },
        "description": "Optional initial task list for the child thread"
      },
      "wait_for_completion": {
        "type": "boolean",
        "description": "Whether to pause current thread until child completes",
        "default": false
      }
    },
    "required": ["goal"]
  }
}
```

### complete_thread
Mark the current thread as completed with a result.

```json
{
  "name": "complete_thread",
  "description": "Mark the current thread as completed",
  "parameters": {
    "type": "object",
    "properties": {
      "status": {
        "type": "string",
        "enum": ["completed", "failed"],
        "description": "The completion status"
      },
      "result_summary": {
        "type": "string",
        "description": "A concise summary of what was accomplished or why it failed"
      },
      "result_data": {
        "type": "object",
        "description": "Optional structured data to return to parent thread"
      }
    },
    "required": ["status", "result_summary"]
  }
}
```

## Context Tools

### get_parent_context
Get context about the parent thread and branching point.

```json
{
  "name": "get_parent_context",
  "description": "Get information about the parent thread and why this thread was created",
  "parameters": {}
}
```

### get_child_threads
List child threads spawned from the current thread.

```json
{
  "name": "get_child_threads",
  "description": "Get status of child threads spawned from current thread",
  "parameters": {
    "type": "object",
    "properties": {
      "include_completed": {
        "type": "boolean",
        "description": "Whether to include completed child threads",
        "default": true
      }
    }
  }
}
```


## Usage Examples

### Starting a Complex Task
1. Agent receives goal: "Implement user authentication"
2. Calls `update_tasks` with breakdown of sub-tasks
3. Spawns child threads for parallel work:
   - `spawn_child_thread` for "Design database schema"
   - `spawn_child_thread` for "Research OAuth providers"
4. Continues with main thread tasks while children run

### Checking Progress
1. Agent calls `get_current_thread_info` to see current tasks
2. Calls `get_child_threads` to check on spawned threads
3. Updates task statuses based on completed work

### Completing Work
1. Agent finishes all tasks
2. Calls `complete_thread` with summary of accomplishments
3. Parent thread receives `thread_result` Stitch with the summary
4. Parent continues processing with child's results