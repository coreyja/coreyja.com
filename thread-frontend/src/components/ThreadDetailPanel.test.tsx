import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { ThreadDetailPanel } from './ThreadDetailPanel'
import { Thread, Stitch } from '../types'

describe('ThreadDetailPanel', () => {
  const mockOnClose = vi.fn()

  const baseThread: Thread = {
    thread_id: '123e4567-e89b-12d3-a456-426614174000',
    parent_thread_id: null,
    branching_stitch_id: null,
    goal: 'Test thread goal',
    tasks: [],
    status: 'completed',
    result: null,
    pending_child_results: [],
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z'
  }

  const baseStitch: Stitch = {
    stitch_id: '223e4567-e89b-12d3-a456-426614174000',
    thread_id: '123e4567-e89b-12d3-a456-426614174000',
    previous_stitch_id: null,
    stitch_type: 'tool_call',
    llm_request: null,
    llm_response: null,
    tool_name: 'database_query',
    tool_input: null,
    tool_output: null,
    child_thread_id: null,
    thread_result_summary: null,
    created_at: '2024-01-01T00:00:00Z'
  }

  beforeEach(() => {
    mockOnClose.mockClear()
  })

  it('renders nothing when no thread or stitch is provided', () => {
    const { container } = render(<ThreadDetailPanel onClose={mockOnClose} />)
    expect(container.firstChild).toBeNull()
  })

  it('renders thread details when thread is provided', () => {
    render(<ThreadDetailPanel thread={baseThread} onClose={mockOnClose} />)
    
    expect(screen.getByText('Thread Details')).toBeInTheDocument()
    expect(screen.getByText('123e4567-e89b-12d3-a456-426614174000')).toBeInTheDocument()
    expect(screen.getByText('Test thread goal')).toBeInTheDocument()
    expect(screen.getByText('completed')).toBeInTheDocument()
  })

  it('renders stitch details when stitch is provided', () => {
    render(<ThreadDetailPanel stitch={baseStitch} onClose={mockOnClose} />)
    
    expect(screen.getByText('Stitch Details')).toBeInTheDocument()
    expect(screen.getByText('223e4567-e89b-12d3-a456-426614174000')).toBeInTheDocument()
    expect(screen.getByText('tool_call')).toBeInTheDocument()
    expect(screen.getByText('database_query')).toBeInTheDocument()
  })

  it('calls onClose when close button is clicked', () => {
    render(<ThreadDetailPanel thread={baseThread} onClose={mockOnClose} />)
    
    const closeButton = screen.getByRole('button', { name: '×' })
    fireEvent.click(closeButton)
    
    expect(mockOnClose).toHaveBeenCalledTimes(1)
  })

  it('renders thread tasks when present', () => {
    const threadWithTasks = {
      ...baseThread,
      tasks: [
        { status: 'completed', description: 'Task 1' },
        { status: 'in_progress', description: 'Task 2' },
        { status: 'pending', description: 'Task 3' }
      ]
    }
    render(<ThreadDetailPanel thread={threadWithTasks} onClose={mockOnClose} />)
    
    expect(screen.getByText('Tasks:')).toBeInTheDocument()
    expect(screen.getByText('[completed]')).toBeInTheDocument()
    expect(screen.getByText('Task 1')).toBeInTheDocument()
    expect(screen.getByText('[in_progress]')).toBeInTheDocument()
    expect(screen.getByText('Task 2')).toBeInTheDocument()
    expect(screen.getByText('[pending]')).toBeInTheDocument()
    expect(screen.getByText('Task 3')).toBeInTheDocument()
  })

  it('renders thread result when present', () => {
    const threadWithResult = {
      ...baseThread,
      result: { success: true, data: 'Result data' }
    }
    render(<ThreadDetailPanel thread={threadWithResult} onClose={mockOnClose} />)
    
    expect(screen.getByText('Result:')).toBeInTheDocument()
    expect(screen.getByText(/success/)).toBeInTheDocument()
    expect(screen.getByText(/Result data/)).toBeInTheDocument()
  })

  it('renders stitch tool details when present', () => {
    const stitchWithToolData = {
      ...baseStitch,
      tool_input: { query: 'SELECT * FROM users' },
      tool_output: { rows: 5, status: 'success' }
    }
    render(<ThreadDetailPanel stitch={stitchWithToolData} onClose={mockOnClose} />)
    
    expect(screen.getByText('Tool Input:')).toBeInTheDocument()
    expect(screen.getByText(/SELECT \* FROM users/)).toBeInTheDocument()
    expect(screen.getByText('Tool Output:')).toBeInTheDocument()
    expect(screen.getByText(/rows/)).toBeInTheDocument()
    expect(screen.getByText(/success/)).toBeInTheDocument()
  })

  it('renders LLM request and response when present', () => {
    const stitchWithLLM = {
      ...baseStitch,
      stitch_type: 'llm_call' as const,
      llm_request: { prompt: 'Test prompt' },
      llm_response: { completion: 'Test response' }
    }
    render(<ThreadDetailPanel stitch={stitchWithLLM} onClose={mockOnClose} />)
    
    expect(screen.getByText('LLM Request:')).toBeInTheDocument()
    expect(screen.getByText(/Test prompt/)).toBeInTheDocument()
    expect(screen.getByText('LLM Response:')).toBeInTheDocument()
    expect(screen.getByText(/Test response/)).toBeInTheDocument()
  })

  it('does not render optional fields when not present', () => {
    render(<ThreadDetailPanel thread={baseThread} onClose={mockOnClose} />)
    
    expect(screen.queryByText('Tasks:')).not.toBeInTheDocument()
    expect(screen.queryByText('Result:')).not.toBeInTheDocument()
  })

  it('applies correct styling to the panel', () => {
    render(<ThreadDetailPanel thread={baseThread} onClose={mockOnClose} />)
    
    const panel = screen.getByText('Thread Details').parentElement!.parentElement!
    expect(panel).toHaveStyle({
      position: 'absolute',
      right: '0',
      top: '0',
      bottom: '0',
      width: '400px',
      background: 'white',
      borderLeft: '1px solid #ccc',
      padding: '20px',
      overflowY: 'auto',
      zIndex: '10'
    })
  })

  it('applies correct color coding to task statuses', () => {
    const threadWithTasks = {
      ...baseThread,
      tasks: [
        { status: 'completed', description: 'Completed task' },
        { status: 'in_progress', description: 'In progress task' },
        { status: 'pending', description: 'Pending task' }
      ]
    }
    render(<ThreadDetailPanel thread={threadWithTasks} onClose={mockOnClose} />)
    
    const completedStatus = screen.getByText('[completed]')
    expect(completedStatus).toHaveStyle({ color: 'rgb(0, 128, 0)' }) // green
    
    const inProgressStatus = screen.getByText('[in_progress]')
    expect(inProgressStatus).toHaveStyle({ color: 'rgb(255, 165, 0)' }) // orange
    
    const pendingStatus = screen.getByText('[pending]')
    expect(pendingStatus).toHaveStyle({ color: 'rgb(128, 128, 128)' }) // gray
  })
})