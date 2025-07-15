import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { StitchNode } from './StitchNode'
import { Stitch } from '../types'

// Mock React Flow components
vi.mock('@xyflow/react', () => ({
  Handle: vi.fn(({ children }) => <div data-testid="handle">{children}</div>),
  Position: {
    Top: 'top',
    Bottom: 'bottom',
  },
}))

describe('StitchNode', () => {
  const mockOnClick = vi.fn()

  const baseStitch: Stitch = {
    stitch_id: '123e4567-e89b-12d3-a456-426614174000',
    thread_id: '223e4567-e89b-12d3-a456-426614174000',
    previous_stitch_id: null,
    stitch_type: 'llm_call',
    llm_request: null,
    llm_response: null,
    tool_name: null,
    tool_input: null,
    tool_output: null,
    child_thread_id: null,
    thread_result_summary: null,
    created_at: '2024-01-01T00:00:00Z'
  }

  const createNodeData = (stitch: Stitch) => ({
    stitch,
    onClick: mockOnClick
  })

  beforeEach(() => {
    mockOnClick.mockClear()
  })

  it('renders llm_call stitch with robot icon', () => {
    render(<StitchNode data={createNodeData(baseStitch)} />)
    
    expect(screen.getByText('🤖')).toBeInTheDocument()
    expect(screen.getByText('llm call')).toBeInTheDocument()
  })

  it('renders tool_call stitch with wrench icon', () => {
    const toolCallStitch = { ...baseStitch, stitch_type: 'tool_call' as const }
    render(<StitchNode data={createNodeData(toolCallStitch)} />)
    
    expect(screen.getByText('🔧')).toBeInTheDocument()
    expect(screen.getByText('tool call')).toBeInTheDocument()
  })

  it('renders thread_result stitch with chart icon', () => {
    const threadResultStitch = { ...baseStitch, stitch_type: 'thread_result' as const }
    render(<StitchNode data={createNodeData(threadResultStitch)} />)
    
    expect(screen.getByText('📊')).toBeInTheDocument()
    expect(screen.getByText('thread result')).toBeInTheDocument()
  })

  it('displays tool name when present', () => {
    const stitchWithTool = { 
      ...baseStitch, 
      stitch_type: 'tool_call' as const,
      tool_name: 'database_query' 
    }
    render(<StitchNode data={createNodeData(stitchWithTool)} />)
    
    expect(screen.getByText('database_query')).toBeInTheDocument()
  })

  it('does not display tool name when null', () => {
    render(<StitchNode data={createNodeData(baseStitch)} />)
    
    expect(screen.queryByText('database_query')).not.toBeInTheDocument()
  })

  it('displays truncated thread result summary when present', () => {
    const stitchWithSummary = { 
      ...baseStitch, 
      stitch_type: 'thread_result' as const,
      thread_result_summary: 'This is a very long thread result summary that should be truncated after fifty characters'
    }
    const { container } = render(<StitchNode data={createNodeData(stitchWithSummary)} />)
    
    // Find the div containing the summary text
    const summaryDiv = container.querySelector('div[style*="font-size: 10px"]')
    expect(summaryDiv).toBeTruthy()
    expect(summaryDiv!.textContent).toBe('This is a very long thread result summary that sho...')
  })

  it('does not display thread result summary when null', () => {
    render(<StitchNode data={createNodeData(baseStitch)} />)
    
    expect(screen.queryByText(/\.\.\./)).not.toBeInTheDocument()
  })

  it('calls onClick when clicked', () => {
    render(<StitchNode data={createNodeData(baseStitch)} />)
    
    const container = screen.getByText('🤖').parentElement!.parentElement!
    fireEvent.click(container)
    
    expect(mockOnClick).toHaveBeenCalledTimes(1)
    expect(mockOnClick).toHaveBeenCalledWith(baseStitch)
  })

  it('applies correct styling', () => {
    render(<StitchNode data={createNodeData(baseStitch)} />)
    
    const container = screen.getByText('🤖').parentElement!.parentElement!
    expect(container).toHaveStyle({
      background: '#f0f0f0',
      border: '1px solid #ccc',
      borderRadius: '6px',
      padding: '8px',
      minWidth: '150px',
      cursor: 'pointer',
      fontSize: '12px'
    })
  })

  it('renders both tool name and icon for tool_call', () => {
    const toolStitch = {
      ...baseStitch,
      stitch_type: 'tool_call' as const,
      tool_name: 'web_search'
    }
    render(<StitchNode data={createNodeData(toolStitch)} />)
    
    expect(screen.getByText('🔧')).toBeInTheDocument()
    expect(screen.getByText('tool call')).toBeInTheDocument()
    expect(screen.getByText('web_search')).toBeInTheDocument()
  })
})