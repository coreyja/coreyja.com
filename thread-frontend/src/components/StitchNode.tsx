import React from 'react'
import { Handle, Position } from '@xyflow/react'
import { Stitch } from '../types'

interface StitchNodeProps {
  data: {
    stitch: Stitch
    onClick: (stitch: Stitch) => void
  }
}

const stitchIcons = {
  initial_prompt: '💬',
  llm_call: '🤖',
  tool_call: '🔧',
  thread_result: '📊',
  discord_message: '💬',
}

export const StitchNode: React.FC<StitchNodeProps> = ({ data }) => {
  const { stitch, onClick } = data
  const icon = stitchIcons[stitch.stitch_type]

  return (
    <div
      style={{
        background: '#f0f0f0',
        border: '1px solid #ccc',
        borderRadius: '6px',
        padding: '8px',
        minWidth: '150px',
        cursor: 'pointer',
        fontSize: '12px',
      }}
      onClick={() => onClick(stitch)}
    >
      <Handle type="target" position={Position.Top} />
      <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
        <span>{icon}</span>
        <span>{stitch.stitch_type.replace('_', ' ')}</span>
      </div>
      {stitch.tool_name && (
        <div style={{ fontSize: '10px', color: '#666', marginTop: '2px' }}>{stitch.tool_name}</div>
      )}
      {stitch.thread_result_summary && (
        <div style={{ fontSize: '10px', color: '#666', marginTop: '2px' }}>
          {stitch.thread_result_summary.substring(0, 50)}...
        </div>
      )}
      <Handle type="source" position={Position.Bottom} />
    </div>
  )
}
