import React, { useState, useEffect, useCallback } from 'react'
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
import { threadsApi } from '../api/threads'
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
  const [selectedThread, setSelectedThread] = useState<Thread | undefined>()
  const [selectedStitch, setSelectedStitch] = useState<Stitch | undefined>()
  const [isLoading, setIsLoading] = useState(true)

  const fetchThreads = useCallback(async () => {
    try {
      const threads = await threadsApi.listThreads()
      const newNodes = [] as Node[]
      const newEdges = [] as Edge[]

      // Create nodes for threads
      threads.forEach((thread, index) => {
        newNodes.push({
          id: `thread-${thread.thread_id}`,
          type: 'thread',
          position: { x: index * 300, y: 0 },
          data: {
            thread,
            onClick: async (t: Thread) => {
              setSelectedThread(t)
              setSelectedStitch(undefined)
              // Fetch full thread with stitches
              const fullThread = await threadsApi.getThread(t.thread_id)
              setSelectedThread(fullThread)
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

      setNodes(newNodes)
      setEdges(newEdges)
      setIsLoading(false)
    } catch (error) {
      console.error('Failed to fetch threads:', error)
      setIsLoading(false)
    }
  }, [setNodes, setEdges])

  useEffect(() => {
    fetchThreads()
    const interval = setInterval(fetchThreads, 2000) // Auto-refresh every 2 seconds
    return () => clearInterval(interval)
  }, [fetchThreads])

  const handlePaneClick = useCallback(() => {
    setSelectedThread(undefined)
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
        thread={selectedThread}
        stitch={selectedStitch}
        onClose={() => {
          setSelectedThread(undefined)
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
