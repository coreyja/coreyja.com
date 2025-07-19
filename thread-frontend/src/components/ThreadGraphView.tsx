import React, { useCallback, useMemo } from 'react'
import { ReactFlow, Controls, Background, BackgroundVariant } from '@xyflow/react'
import type { Node, Edge } from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { useNavigate } from '@tanstack/react-router'

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

interface ThreadGraphViewProps {
  threadId?: string
  stitchId?: string
}

export const ThreadGraphView: React.FC<ThreadGraphViewProps> = ({ threadId, stitchId }) => {
  const navigate = useNavigate()

  console.log('threadId', threadId)
  console.log('stitchId', stitchId)

  const { data: recentThreads, isLoading } = useRecentThreads()
  const { data: selectedThreadDetails } = useThread(threadId)
  const { data: selectedThreadChildren } = useThreadChildren(threadId)

  // Debug logging
  console.log('ThreadGraphView Debug:', {
    threadId,
    selectedThreadDetails,
    selectedThreadChildren,
    stitches: selectedThreadDetails?.stitches,
  })
  const { data: parentThreads } = useThreadParents(threadId)

  // Check if the selected thread is a top-level thread
  const isSelectedThreadTopLevel = recentThreads?.some(t => t.thread_id === threadId)

  const { nodesData, edgesData } = useMemo(() => {
    const newNodes = [] as Node[]
    const newEdges = [] as Edge[]

    // Only show recent top-level threads if no inner thread is selected
    if (recentThreads && !threadId) {
      recentThreads.forEach((thread, index) => {
        // Fade out the thread if it's not the selected one and a thread is selected
        const opacity = threadId && thread.thread_id !== threadId ? 0.3 : 1
        const threadNode = {
          id: `thread-${thread.thread_id}`,
          type: 'thread',
          position: { x: index * 300, y: 0 },
          data: {
            thread,
            opacity,
            onClick: async (t: Thread | ThreadWithCounts) => {
              await navigate({ to: '.', search: { thread: t.thread_id } })
            },
          },
        }
        newNodes.push(threadNode)
      })
    }

    // Add child thread nodes and edges for selected thread
    if (
      threadId &&
      selectedThreadChildren &&
      selectedThreadDetails &&
      selectedThreadDetails.stitches
    ) {
      selectedThreadChildren.forEach((childThread, index) => {
        const parentStitch = selectedThreadDetails.stitches.find(
          s => s.stitch_id === childThread.branching_stitch_id
        )

        // Add child thread node
        const childNode = {
          id: `thread-${childThread.thread_id}`,
          type: 'thread',
          position: { x: 600, y: 100 + index * 150 },
          data: {
            thread: childThread,
            opacity: 0.8,
            onClick: (t: Thread | ThreadWithCounts) => {
              navigate({ to: '.', search: { thread: t.thread_id } })
            },
          },
        }
        newNodes.push(childNode)

        if (parentStitch) {
          // Add edge from parent stitch to child thread
          newEdges.push({
            id: `edge-stitch-${parentStitch.stitch_id}-thread-${childThread.thread_id}`,
            source: `stitch-${parentStitch.stitch_id}`,
            target: `thread-${childThread.thread_id}`,
            animated: true,
            style: { stroke: '#FF9800', strokeWidth: 2, strokeDasharray: '5,5' },
            type: 'smoothstep',
            label: 'spawns',
            labelStyle: { fontSize: 10, fontWeight: 700 },
          })
        }
      })
    }

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
              navigate({ to: '.', search: { thread: t.thread_id } })
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

      // Add edge from last parent to thread
      if (parentThreads && parentThreads.length > 0) {
        const lastParent = parentThreads[parentThreads.length - 1]
        newEdges.push({
          id: `parent-edge-${lastParent.thread_id}-${threadId}`,
          source: `parent-thread-${lastParent.thread_id}`,
          target: `thread-${threadId}`,
          style: { stroke: '#888', strokeWidth: 2, opacity: 0.5 },
          animated: false,
        })
      }
    }

    // Add the selected thread node when threadId is set
    if (threadId && selectedThreadDetails) {
      const threadNode = {
        id: `thread-${threadId}`,
        type: 'thread',
        position: { x: 300, y: 0 },
        data: {
          thread: {
            ...selectedThreadDetails,
            stitch_count: selectedThreadDetails.stitches?.length || 0,
            children_count: selectedThreadChildren?.length || 0,
          },
          opacity: 1,
          onClick: (t: Thread | ThreadWithCounts) => {
            navigate({ to: '.', search: { thread: t.thread_id } })
          },
        },
      }
      newNodes.push(threadNode)

      // Add stitches for the selected thread
      if (selectedThreadDetails.stitches) {
        selectedThreadDetails.stitches.forEach((stitch, stitchIndex) => {
          const stitchY = 100 + stitchIndex * 80
          const stitchNode = {
            id: `stitch-${stitch.stitch_id}`,
            type: 'stitch',
            position: { x: 300, y: stitchY },
            data: {
              stitch,
              onClick: (s: Stitch) => {
                navigate({
                  to: '.',
                  search: { thread: threadId, stitch: s.stitch_id },
                })
              },
            },
          }
          newNodes.push(stitchNode)

          // Add edge from thread to first stitch
          if (stitchIndex === 0) {
            newEdges.push({
              id: `edge-thread-${threadId}-stitch-${stitch.stitch_id}`,
              source: `thread-${threadId}`,
              target: `stitch-${stitch.stitch_id}`,
              style: { stroke: '#4CAF50', strokeWidth: 2 },
              animated: true,
            })
          }

          // Add edges between consecutive stitches
          if (stitchIndex > 0) {
            const previousStitch = selectedThreadDetails?.stitches[stitchIndex - 1]
            if (previousStitch) {
              newEdges.push({
                id: `edge-stitch-${previousStitch.stitch_id}-${stitch.stitch_id}`,
                source: `stitch-${previousStitch.stitch_id}`,
                target: `stitch-${stitch.stitch_id}`,
                style: { stroke: '#2196F3', strokeWidth: 1.5 },
              })
            }
          }
        })
      }
    }

    // Add stitch nodes for the selected top-level thread
    if (
      selectedThreadDetails &&
      selectedThreadDetails.stitches &&
      isSelectedThreadTopLevel &&
      !threadId
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
              navigate({
                to: '.',
                search: { thread: threadId, stitch: s.stitch_id },
              })
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
                opacity: threadId === childThread.thread_id ? 1 : 0.7,
                onClick: (t: Thread) => {
                  navigate({ to: '.', search: { thread: t.thread_id } })
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
    threadId,
    isSelectedThreadTopLevel,
    parentThreads,
    navigate,
  ])

  const handlePaneClick = useCallback(() => {
    navigate({ to: '.', search: {} })
  }, [navigate])

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
        stitch={
          stitchId
            ? selectedThreadDetails?.stitches?.find(s => s.stitch_id === stitchId)
            : undefined
        }
        onClose={() => {
          navigate({ to: '.', search: {} })
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
