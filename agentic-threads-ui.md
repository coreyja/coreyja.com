# Agentic Threads UI Design

## Overview

Web-based visualization for exploring thread hierarchies, with zoom/pan navigation and interactive elements to inspect thread details, stitches, and relationships.

## Technology Stack Recommendation

### React Flow (https://reactflow.dev/)
**Why React Flow:**
- Built for node-based graph UIs with built-in zoom/pan
- Handles complex hierarchical layouts out of the box
- React-based (assuming you're using React)
- Excellent performance with large graphs
- Built-in minimap, controls, and interactions
- Easy to customize nodes and edges

**Alternative Options:**
- **D3.js**: More flexible but requires more implementation
- **Cytoscape.js**: Good for complex graphs but heavier
- **Vis.js**: Simple but less React-friendly
- **Mermaid**: Great for static diagrams, not interactive enough

## UI Components

### Main Graph View

```typescript
interface ThreadNode {
  id: string; // thread_id
  type: 'thread';
  position: { x: number; y: number };
  data: {
    goal: string;
    status: 'pending' | 'running' | 'waiting' | 'completed' | 'failed';
    taskCount: number;
    completedTaskCount: number;
    stitchCount: number;
  };
}

interface StitchNode {
  id: string; // stitch_id
  type: 'stitch';
  position: { x: number; y: number };
  data: {
    stitchType: 'llm_call' | 'tool_call' | 'thread_result';
    summary: string; // First 100 chars of content
    timestamp: string;
  };
}
```

### Visual Design

```
Thread Node (Rounded Rectangle):
┌─────────────────────────┐
│ 🎯 Goal (truncated)     │
│ ━━━━━━━━━━━━━          │ <- Progress bar
│ 3/5 tasks • 12 stitches │
└─────────────────────────┘

Stitch Node (Circle for tools, Square for LLM):
  ○ Tool: spawn_child
  □ LLM: "I'll create..."
  ◇ Result: Child complete

Edges:
- Solid line: Thread to child thread
- Dotted line: Stitch to next stitch
- Colored by status (green=complete, yellow=running, red=failed)
```

### Layout Strategy

1. **Hierarchical Layout**: Threads arranged in tree structure
2. **Collapsible Threads**: Click to expand/collapse point details
3. **Smart Positioning**: Child threads branch downward and rightward
4. **Auto-layout**: Using React Flow's dagre layout algorithm

## Interactive Features

### Thread Node Interactions
- **Click**: Open detail panel with full goal, tasks, and metadata
- **Double-click**: Expand/collapse to show Stitches
- **Hover**: Show tooltip with status and timing info
- **Right-click**: Context menu (Re-run, Cancel, View logs)

### Stitch Node Interactions
- **Click**: Show full LLM conversation or tool input/output
- **Hover**: Preview content in tooltip
- **Badge**: Shows execution time

### Navigation Controls
- **Zoom**: Mouse wheel or pinch
- **Pan**: Click and drag
- **Fit view**: Button to center graph
- **Minimap**: Overview in corner
- **Search**: Find threads by goal or ID

## Detail Panels

### Thread Detail Panel (Sidebar)
```
┌─ Thread: auth-implementation-xyz ─┐
│ Goal: Implement user auth         │
│ Status: Running ⚡                │
│ Created: 2024-01-15 10:30am      │
│                                   │
│ Tasks:                            │
│ ✓ Design schema                   │
│ ⚡ Create API endpoints            │
│ ○ Add frontend forms              │
│                                   │
│ Child Threads (2):                │
│ ✓ research-oauth-providers        │
│ ⚡ design-db-schema               │
└───────────────────────────────────┘
```

### Stitch Detail Modal
```
┌─ LLM Call ──────────────────────┐
│ Timestamp: 10:31:42             │
│                                 │
│ Request:                        │
│ ┌─────────────────────────────┐ │
│ │ Goal: Implement user auth   │ │
│ │ Context: ...                │ │
│ └─────────────────────────────┘ │
│                                 │
│ Response:                       │
│ ┌─────────────────────────────┐ │
│ │ I'll break this down into   │ │
│ │ several tasks...            │ │
│ └─────────────────────────────┘ │
└─────────────────────────────────┘
```

## API Endpoints Needed

```typescript
// Get full thread tree
GET /api/threads/:id/tree
Response: {
  thread: Thread,
  stitches: Stitch[],
  children: ThreadTree[] // Recursive
}

// Get thread list for initial view
GET /api/threads?root_only=true&limit=20&status=active

// Get specific stitch details
GET /api/stitches/:id

// Real-time updates via WebSocket
WS /api/threads/subscribe
```

## Implementation Plan

1. **Set up React Flow**
   ```bash
   npm install reactflow
   ```

2. **Create custom node components**
   - ThreadNode component with progress visualization
   - StitchNode component with type indicators
   - Custom edge with status colors

3. **Implement auto-layout**
   - Use dagre for hierarchical layout
   - Calculate positions based on tree depth

4. **Add interactivity**
   - Click handlers for details
   - Expand/collapse logic
   - Zoom controls

5. **Connect to backend**
   - Fetch thread trees
   - WebSocket for live updates
   - Optimistic UI updates

## Example React Component Structure

```tsx
// Main component
<ThreadGraphView>
  <ReactFlow
    nodes={[...threadNodes, ...stitchNodes]}
    edges={edges}
    nodeTypes={{ thread: ThreadNode, stitch: StitchNode }}
  >
    <Controls />
    <MiniMap />
    <Background />
  </ReactFlow>
  <ThreadDetailPanel thread={selectedThread} />
</ThreadGraphView>
```

This gives you a powerful, interactive visualization with minimal custom implementation thanks to React Flow's built-in features.