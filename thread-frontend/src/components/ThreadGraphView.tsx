import React, { useState, useCallback, useMemo } from 'react'
import { ReactFlow, Controls, Background, BackgroundVariant } from '@xyflow/react'
import type { Node, Edge } from '@xyflow/react'
import '@xyflow/react/dist/style.css'

import { Thread, Stitch, ThreadWithCounts } from '../types'
import {
  useRecentThreads,
  useThread,
  useThreadChildren,
  useThreadParents,
} from '../hooks/useThreads'
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
  const [selectedInnerThreadId, setSelectedInnerThreadId] = useState<string>()

  const { data: recentThreads, isLoading } = useRecentThreads()
  const { data: selectedThreadDetails } = useThread(selectedThreadId)
  const { data: selectedThreadChildren } = useThreadChildren(selectedThreadId)
  const { data: innerThreadDetails } = useThread(selectedInnerThreadId)
  const { data: parentThreads } = useThreadParents(selectedInnerThreadId)

  // Check if the selected thread is a top-level thread
  const isSelectedThreadTopLevel = recentThreads?.some(t => t.thread_id === selectedThreadId)

  const { nodesData, edgesData } = useMemo(() => {
    const newNodes = [] as Node[]
    const newEdges = [] as Edge[]

    // Only show recent top-level threads if no inner thread is selected
    if (recentThreads && !selectedInnerThreadId) {
      recentThreads.forEach((thread, index) => {
        // Fade out the thread if it's not the selected one and a thread is selected
        const opacity = selectedThreadId && thread.thread_id !== selectedThreadId ? 0.3 : 1
        const threadNode = {
          id: `thread-${thread.thread_id}`,
          type: 'thread',
          position: { x: index * 300, y: 0 },
          data: {
            thread,
            opacity,
            onClick: (t: Thread | ThreadWithCounts) => {
              setSelectedThreadId(t.thread_id)
              setSelectedInnerThreadId(undefined)
              setSelectedStitch(undefined)
            },
          },
        }
        newNodes.push(threadNode)
      })
    }

    // Add parent-child edges for selected thread's children
    if (selectedThreadChildren && selectedThreadDetails && selectedThreadDetails.stitches) {
      selectedThreadChildren.forEach(childThread => {
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

    // Show the selected inner thread if one is selected
    if (selectedInnerThreadId && innerThreadDetails) {
      // Add parent thread stack
      if (parentThreads) {
        parentThreads.forEach((parentThread, index) => {
          const parentNode = {
            id: `parent-thread-${parentThread.thread_id}`,
            type: 'thread',
            position: { x: 300, y: -150 * (parentThreads.length - index) },
            data: {
              thread: parentThread,
              opacity: 0.5,
              onClick: (t: Thread | ThreadWithCounts) => {
                setSelectedThreadId(t.thread_id)
                setSelectedInnerThreadId(undefined)
                setSelectedStitch(undefined)
              },
            },
          }
          newNodes.push(parentNode)

          // Add edge from parent to child
          if (index < parentThreads.length - 1) {
            newEdges.push({
              id: `parent-edge-${parentThread.thread_id}-${parentThreads[index + 1].thread_id}`,
              source: `parent-thread-${parentThread.thread_id}`,
              target: `parent-thread-${parentThreads[index + 1].thread_id}`,
              style: { stroke: '#888', strokeWidth: 2, opacity: 0.5 },
              animated: false,
            })
          }
        })
      }

      // Add edge from last parent to inner thread
      if (parentThreads && parentThreads.length > 0) {
        const lastParent = parentThreads[parentThreads.length - 1]
        newEdges.push({
          id: `parent-edge-${lastParent.thread_id}-${innerThreadDetails.thread_id}`,
          source: `parent-thread-${lastParent.thread_id}`,
          target: `thread-${innerThreadDetails.thread_id}`,
          style: { stroke: '#888', strokeWidth: 2, opacity: 0.5 },
          animated: false,
        })
      }

      const innerThreadNode = {
        id: `thread-${innerThreadDetails.thread_id}`,
        type: 'thread',
        position: { x: 300, y: 0 },
        data: {
          thread: innerThreadDetails,
          opacity: 1,
          onClick: (t: Thread | ThreadWithCounts) => {
            setSelectedThreadId(t.thread_id)
            setSelectedInnerThreadId(t.thread_id)
            setSelectedStitch(undefined)
          },
        },
      }
      newNodes.push(innerThreadNode)

      // Add stitches for the inner thread
      if (innerThreadDetails.stitches) {
        innerThreadDetails.stitches.forEach((stitch, stitchIndex) => {
          const stitchY = 100 + stitchIndex * 80
          const stitchNode = {
            id: `stitch-${stitch.stitch_id}`,
            type: 'stitch',
            position: { x: 300, y: stitchY },
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
              id: `edge-thread-${innerThreadDetails.thread_id}-stitch-${stitch.stitch_id}`,
              source: `thread-${innerThreadDetails.thread_id}`,
              target: `stitch-${stitch.stitch_id}`,
              style: { stroke: '#4CAF50', strokeWidth: 2 },
              animated: true,
            })
          }

          // Add edges between consecutive stitches
          if (stitchIndex > 0) {
            const previousStitch = innerThreadDetails.stitches[stitchIndex - 1]
            newEdges.push({
              id: `edge-stitch-${previousStitch.stitch_id}-${stitch.stitch_id}`,
              source: `stitch-${previousStitch.stitch_id}`,
              target: `stitch-${stitch.stitch_id}`,
              style: { stroke: '#2196F3', strokeWidth: 1.5 },
            })
          }
        })
      }
    }

    // Add stitch nodes for the selected top-level thread
    if (
      selectedThreadDetails &&
      selectedThreadDetails.stitches &&
      isSelectedThreadTopLevel &&
      !selectedInnerThreadId
    ) {
      const threadIndex =
        recentThreads?.findIndex(t => t.thread_id === selectedThreadDetails.thread_id) ?? 0
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
        if (stitch.child_thread_id && selectedThreadChildren) {
          const childThread = selectedThreadChildren.find(
            t => t.thread_id === stitch.child_thread_id
          )
          if (childThread) {
            const childThreadNode = {
              id: `thread-${childThread.thread_id}-from-stitch`,
              type: 'thread',
              position: { x: threadX + 200, y: stitchY },
              data: {
                thread: childThread,
                opacity: selectedInnerThreadId === childThread.thread_id ? 1 : 0.7,
                onClick: (t: Thread) => {
                  setSelectedThreadId(t.thread_id)
                  setSelectedInnerThreadId(t.thread_id)
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
  }, [
    recentThreads,
    selectedThreadDetails,
    selectedThreadChildren,
    selectedInnerThreadId,
    innerThreadDetails,
    isSelectedThreadTopLevel,
    selectedThreadId,
    parentThreads,
  ])

  const handlePaneClick = useCallback(() => {
    setSelectedThreadId(undefined)
    setSelectedInnerThreadId(undefined)
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
        thread={selectedThreadDetails || innerThreadDetails}
        stitch={selectedStitch}
        onClose={() => {
          setSelectedThreadId(undefined)
          setSelectedInnerThreadId(undefined)
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
