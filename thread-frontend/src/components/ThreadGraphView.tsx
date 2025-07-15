import React, { useState, useEffect, useCallback, useMemo } from 'react'
import {
  ReactFlow,
  useNodesState,
  useEdgesState,
  Controls,
  Background,
  BackgroundVariant,
} from '@xyflow/react'
import type { Node, Edge } from '@xyflow/react'
import '@xyflow/react/dist/style.css'

import { Thread, Stitch } from '../types'
import { useThreads, useThread } from '../hooks/useThreads'
import { ThreadNode } from './ThreadNode'
import { StitchNode } from './StitchNode'
import { ThreadDetailPanel } from './ThreadDetailPanel'

const nodeTypes = {
  thread: ThreadNode,
  stitch: StitchNode,
}

export const ThreadGraphView: React.FC = () => {
  const [nodes, setNodes, onNodesChange] = useNodesState([] as Node[])
  const [edges, setEdges, onEdgesChange] = useEdgesState([] as Edge[])
  const [selectedThreadId, setSelectedThreadId] = useState<string | undefined>()
  const [selectedStitch, setSelectedStitch] = useState<Stitch | undefined>()

  const { data: threads, isLoading } = useThreads()
  const { data: selectedThreadDetails } = useThread(selectedThreadId)

  const { nodesData, edgesData } = useMemo(() => {
    const newNodes = [] as Node[]
    const newEdges = [] as Edge[]

    if (threads) {
      threads.forEach((thread, index) => {
        newNodes.push({
          id: `thread-${thread.thread_id}`,
          type: 'thread',
          position: { x: index * 300, y: 0 },
          data: {
            thread,
            onClick: (t: Thread) => {
              setSelectedThreadId(t.thread_id)
              setSelectedStitch(undefined)
            },
          },
        })

        // Add edges for parent-child relationships
        if (thread.parent_thread_id) {
          newEdges.push({
            id: `edge-${thread.parent_thread_id}-${thread.thread_id}`,
            source: `thread-${thread.parent_thread_id}`,
            target: `thread-${thread.thread_id}`,
          })
        }
      })
    }

    return { nodesData: newNodes, edgesData: newEdges }
  }, [threads])

  useEffect(() => {
    setNodes(nodesData)
    setEdges(edgesData)
  }, [nodesData, edgesData, setNodes, setEdges])

  const handlePaneClick = useCallback(() => {
    setSelectedThreadId(undefined)
    setSelectedStitch(undefined)
  }, [])

  if (isLoading) {
    return (
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          height: '100vh',
        }}
      >
        Loading threads...
      </div>
    )
  }

  return (
    <div style={{ width: '100vw', height: '100vh', position: 'relative' }}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onPaneClick={handlePaneClick}
        nodeTypes={nodeTypes}
        fitView
      >
        <Controls />
        <Background variant={BackgroundVariant.Dots} gap={12} size={1} />
      </ReactFlow>

      <ThreadDetailPanel
        thread={selectedThreadDetails}
        stitch={selectedStitch}
        onClose={() => {
          setSelectedThreadId(undefined)
          setSelectedStitch(undefined)
        }}
      />

      <div
        style={{
          position: 'absolute',
          top: '10px',
          left: '10px',
          background: 'white',
          padding: '10px',
          borderRadius: '8px',
          boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
        }}
      >
        <h3 style={{ margin: '0 0 10px 0' }}>Agentic Threads Visualization</h3>
        <div style={{ fontSize: '12px', color: '#666' }}>
          <div>• Click a thread to see details</div>
          <div>• Auto-refreshes every 2 seconds</div>
        </div>
      </div>
    </div>
  )
}
