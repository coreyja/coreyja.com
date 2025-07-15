# Agentic Threads Implementation - Vertical Slices

This document outlines the incremental vertical slices for implementing the Agentic Threads system. Each slice delivers a working piece of functionality.

## Slice 1: Foundation - Database, Models, API, and MVP UI

**Goal**: Establish the full foundation with database, basic API, and a working UI to visualize threads

### Database Tasks

1. **Create database migration** for threads and stitches tables
   - Add migration file with thread and stitch tables as specified in schema
   - Include all indexes and constraints
   - Add up/down migrations

2. **Implement Thread and Stitch models** with SQLx
   - Create `db/src/agentic_threads/mod.rs`
   - Define Thread struct with all fields
   - Define Stitch struct with all fields
   - Add basic CRUD operations (create, get_by_id, update_status)

### API Tasks

3. **Create API endpoints**
   - GET /api/threads - List threads
   - GET /api/threads/:id - Get thread with stitches
   - Basic serialization of Thread and Stitch models
   - CORS configuration for separate React app

### UI Tasks (New React Project)

4. **Set up separate React project**
   - Create new folder: `agentic-threads-ui/`
   - Initialize with: `npx create-react-app agentic-threads-ui --template typescript`
   - Install dependencies:
     ```bash
     cd agentic-threads-ui
     npm install reactflow @xyflow/react
     npm install axios
     npm install @types/react @types/react-dom
     ```

5. **Create basic Thread visualization**
   - Configure API base URL (e.g., `http://localhost:3000/api`)
   - Create ThreadGraphView component
   - Fetch and display threads as nodes
   - Basic status colors (pending=gray, running=yellow, completed=green, failed=red)
   - Auto-refresh every 2 seconds to see status changes
   - Simple sidebar to show thread details on click

### Testing Slice 1

- Create test threads directly in database
- Run the UI: `cd agentic-threads-ui && npm start`
- Verify threads appear in the visualization
- Click threads to see details in sidebar

## Slice 2: Basic Job Processing with LLM

**Goal**: Create a working thread processor that can make LLM calls

### Job Processing Tasks

1. **Create ProcessThreadStep job** with basic LLM integration
   - Implement the Job trait for ProcessThreadStep
   - Add logic to find last stitch in thread
   - Make initial LLM call for threads with no stitches
   - Store LLM response as new stitch
   - Re-enqueue job for next step

2. **Add Anthropic client integration**
   - Define system prompt for thread processing
   - Handle tool function definitions in API format

### UI Enhancement Tasks

3. **Update UI to show stitches**
   - Modify API to include stitches in thread response
   - Display stitches as child nodes under threads
   - Show stitch type icons (LLM call, tool call)
   - Click stitch to see full content in sidebar
   - Add visual indication of active processing

## Slice 3: Thread Management Tools

**Goal**: Enable threads to manage their own state and tasks

### Tool Implementation Tasks

1. **Implement basic thread tools**
   - `get_current_thread_info`: Return thread goal, status, tasks
   - `update_tasks`: Update the JSONB tasks field
   - Create tool executor that handles these function calls

2. **Integrate tools into ProcessThreadStep**
   - Parse LLM responses for tool calls
   - Execute requested tools
   - Create tool result stitches
   - Continue conversation with tool results

### UI Enhancement Tasks

3. **Enhance UI for task tracking**
   - Show task progress bar on thread nodes
   - Display task list in thread detail panel
   - Real-time updates as tasks change
   - Color-code tasks by status

## Slice 4: Thread Spawning and Completion

**Goal**: Allow threads to create child threads and report completion

### Thread Lifecycle Tasks

1. **Add thread lifecycle tools**
   - `spawn_child_thread`: Create new child thread
   - `complete_thread`: Mark thread as completed/failed
   - Handle parent_thread_id and branching_stitch_id

2. **Implement NotifyParentThread job**
   - Create job that handles child completion
   - Update parent's pending_child_results
   - Create thread_result stitches
   - Handle parent thread re-enqueueing

### UI Hierarchy Tasks

3. **Update UI for thread hierarchy**
   - Add GET /api/threads/:id/tree endpoint
   - Display parent-child relationships as edges
   - Implement expand/collapse for thread trees
   - Show thread_result stitches with special styling
   - Use dagre layout for automatic positioning

## Slice 5: Cron Integration and Health Monitoring

**Goal**: Demonstrate the system with real use cases and monitoring

### Cron Tasks

1. **Create example cron job**
   - Simple daily standup thread creator
   - Register in cron registry
   - Create thread with specific goal
   - Enqueue ProcessThreadStep for the new thread

2. **Add thread health monitoring**
   - ThreadHealthCheck cron job
   - Find stuck threads (running > 10 min)
   - Reset status and re-enqueue
   - Add observability/logging

### Admin Integration Tasks

3. **Add admin UI integration**
   - Add threads section to admin panel
   - Display thread health metrics
   - Manual thread creation form
   - Ability to restart stuck threads

## UI Project Structure

The separate React project (`agentic-threads-ui/`) will have this structure:

```
agentic-threads-ui/
├── src/
│   ├── components/
│   │   ├── ThreadGraphView.tsx      # Main visualization component
│   │   ├── ThreadNode.tsx           # Custom thread node
│   │   ├── StitchNode.tsx          # Custom stitch node
│   │   ├── ThreadDetailPanel.tsx    # Sidebar for details
│   │   └── Controls.tsx            # Zoom/pan controls
│   ├── api/
│   │   └── threads.ts              # API client functions
│   ├── types/
│   │   └── index.ts                # TypeScript interfaces
│   └── App.tsx                     # Main app component
├── package.json
└── README.md
```

## Development Workflow

1. **Backend Development**
   - Work in the main `coreyja.com` Rust project
   - Run with: `cargo run`
   - API available at: `http://localhost:3000/api`

2. **Frontend Development**
   - Work in the `agentic-threads-ui/` folder
   - Run with: `npm start`
   - UI available at: `http://localhost:3001`
   - Proxy API calls or use CORS

3. **Testing Full Stack**
   - Start backend server
   - Start React dev server
   - Create test threads via SQL or API
   - Watch visualization update in real-time

## Implementation Order

The slices are designed to be implemented in order, with each building on the previous:

1. **Slice 1** provides the complete foundation with immediate visual feedback
2. **Slice 2** creates a minimal working system (threads that talk to LLM)
3. **Slice 3** adds self-management capabilities
4. **Slice 4** enables the key feature of parallel child threads
5. **Slice 5** demonstrates real-world usage and monitoring

## Key Benefits of Combined First Slice

Having database, API, and UI in the first slice provides:

1. **Full Stack Foundation**: Everything needed to visualize the system
2. **Immediate Feedback**: See threads as soon as they're created
3. **Easier Debugging**: Visual representation from day one
4. **Parallel Development**: Frontend and backend can evolve together
5. **Early Demo Capability**: Show working visualization to stakeholders

## Next Steps

1. Create the database migration:
   ```sql
   db/migrations/[timestamp]_create_agentic_threads.up.sql
   ```

2. Set up the React project:
   ```bash
   npx create-react-app agentic-threads-ui --template typescript
   cd agentic-threads-ui
   npm install reactflow axios
   ```

3. Implement the basic models and API endpoints

4. Create the initial ThreadGraphView component

This approach gives you a working, visual system by the end of Slice 1!