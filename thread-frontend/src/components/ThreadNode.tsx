import React from 'react'
import { Handle, Position } from '@xyflow/react'
import { Thread } from '../types'

interface ThreadNodeProps {
  data: {
    thread: Thread
    onClick: (thread: Thread) => void
  }
}

const statusColors = {
  pending: '#9CA3AF', // gray
  running: '#F59E0B', // yellow
  waiting: '#3B82F6', // blue
  completed: '#10B981', // green
  failed: '#EF4444', // red
}

export const ThreadNode: React.FC<ThreadNodeProps> = ({ data }) => {
  const { thread, onClick } = data
  const color = statusColors[thread.status]

  return (
    <div
      style={{
        background: 'white',
        border: `2px solid ${color}`,
        borderRadius: '8px',
        padding: '10px',
        minWidth: '200px',
        cursor: 'pointer',
      }}
      onClick={() => onClick(thread)}
    >
      <Handle type="target" position={Position.Top} />
      <div style={{ fontWeight: 'bold', marginBottom: '4px' }}>Thread</div>
      <div style={{ fontSize: '12px', color: '#666' }}>{thread.goal.substring(0, 50)}...</div>
      <div style={{ fontSize: '10px', color: color, marginTop: '4px' }}>
        {thread.status.toUpperCase()}
      </div>
      {thread.tasks.length > 0 && (
        <div style={{ fontSize: '10px', marginTop: '4px' }}>
          Tasks: {thread.tasks.filter(t => t.status === 'completed').length}/{thread.tasks.length}
        </div>
      )}
      <Handle type="source" position={Position.Bottom} />
    </div>
  )
}
