import React, { useState, useCallback, useMemo } from 'react'
import { ReactFlow, Controls, Background, BackgroundVariant } from '@xyflow/react'
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
  const [selectedThreadId, setSelectedThreadId] = useState<string>()
  const [selectedStitch, setSelectedStitch] = useState<Stitch>()

  const { data: threads, isLoading } = useThreads()
  const { data: selectedThreadDetails } = useThread(selectedThreadId)

  const { nodesData, edgesData } = useMemo(() => {
    const newNodes = [] as Node[]
    const newEdges = [] as Edge[]

    if (threads) {
      threads.forEach((thread, index) => {
        const threadNode = {
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
        }
        newNodes.push(threadNode)
      })
    }

    // Add parent-child edges based on branching relationships
    if (threads) {
      // Create a map of child threads that have branching_stitch_id
      const childThreads = threads.filter(t => t.branching_stitch_id)

      // For the selected thread, check if any of its stitches spawned child threads
      if (selectedThreadDetails && selectedThreadDetails.stitches) {
        childThreads.forEach(childThread => {
          const parentStitch = selectedThreadDetails.stitches.find(
            s => s.stitch_id === childThread.branching_stitch_id
          )
          if (parentStitch) {
            // Add edge from parent thread to child thread
            newEdges.push({
              id: `edge-${selectedThreadDetails.thread_id}-${childThread.thread_id}`,
              source: `thread-${selectedThreadDetails.thread_id}`,
              target: `thread-${childThread.thread_id}`,
              animated: true,
              style: { stroke: '#888', strokeWidth: 2 },
              type: 'smoothstep',
            })
          }
        })
      }
    }

    // Add stitch nodes for the selected thread
    if (selectedThreadDetails && selectedThreadDetails.stitches) {
      const threadIndex =
        threads?.findIndex(t => t.thread_id === selectedThreadDetails.thread_id) ?? 0
      const threadX = threadIndex * 300

      selectedThreadDetails.stitches.forEach((stitch, stitchIndex) => {
        const stitchY = 100 + stitchIndex * 80
        const stitchNode = {
          id: `stitch-${stitch.stitch_id}`,
          type: 'stitch',
          position: { x: threadX, y: stitchY },
          data: {
            stitch,
            onClick: (s: Stitch) => {
              setSelectedStitch(s)
            },
          },
        }
        newNodes.push(stitchNode)

        // Add edge from thread to first stitch
        if (stitchIndex === 0) {
          newEdges.push({
            id: `edge-thread-${selectedThreadDetails.thread_id}-stitch-${stitch.stitch_id}`,
            source: `thread-${selectedThreadDetails.thread_id}`,
            target: `stitch-${stitch.stitch_id}`,
            style: { stroke: '#4CAF50', strokeWidth: 2 },
            animated: true,
          })
        }

        // Add edges between consecutive stitches
        if (stitchIndex > 0) {
          const previousStitch = selectedThreadDetails.stitches[stitchIndex - 1]
          newEdges.push({
            id: `edge-stitch-${previousStitch.stitch_id}-${stitch.stitch_id}`,
            source: `stitch-${previousStitch.stitch_id}`,
            target: `stitch-${stitch.stitch_id}`,
            style: { stroke: '#2196F3', strokeWidth: 1.5 },
          })
        }

        // If this stitch spawned a child thread, show it
        if (stitch.child_thread_id) {
          const childThread = threads?.find(t => t.thread_id === stitch.child_thread_id)
          if (childThread) {
            const childThreadNode = {
              id: `thread-${childThread.thread_id}-from-stitch`,
              type: 'thread',
              position: { x: threadX + 200, y: stitchY },
              data: {
                thread: childThread,
                onClick: (t: Thread) => {
                  setSelectedThreadId(t.thread_id)
                  setSelectedStitch(undefined)
                },
              },
            }
            newNodes.push(childThreadNode)

            // Add edge from stitch to child thread
            newEdges.push({
              id: `edge-stitch-${stitch.stitch_id}-thread-${childThread.thread_id}`,
              source: `stitch-${stitch.stitch_id}`,
              target: `thread-${childThread.thread_id}-from-stitch`,
              style: { stroke: '#FF9800', strokeWidth: 2, strokeDasharray: '5,5' },
              animated: true,
              label: 'spawns',
              labelStyle: { fontSize: 10, fontWeight: 700 },
            })
          }
        }
      })
    }

    return { nodesData: newNodes, edgesData: newEdges }
  }, [threads, selectedThreadDetails])

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
        nodes={nodesData}
        edges={edgesData}
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
