import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { ThreadNode } from './ThreadNode'
import { Thread } from '../types'

// Mock React Flow components
vi.mock('@xyflow/react', () => ({
  Handle: vi.fn(({ children }) => <div data-testid="handle">{children}</div>),
  Position: {
    Top: 'top',
    Bottom: 'bottom',
  },
}))

describe('ThreadNode', () => {
  const mockOnClick = vi.fn()

  const baseThread: Thread = {
    thread_id: '123e4567-e89b-12d3-a456-426614174000',
    parent_thread_id: null,
    branching_stitch_id: null,
    goal: 'Test thread goal',
    tasks: [],
    status: 'pending',
    result: null,
    pending_child_results: [],
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z'
  }

  const createNodeData = (thread: Thread) => ({
    thread,
    onClick: mockOnClick
  })

  beforeEach(() => {
    mockOnClick.mockClear()
  })

  it('renders thread with pending status', () => {
    render(<ThreadNode data={createNodeData(baseThread)} />)
    
    expect(screen.getByText('Thread')).toBeInTheDocument()
    expect(screen.getByText('Test thread goal...')).toBeInTheDocument()
    expect(screen.getByText('PENDING')).toBeInTheDocument()
    
    const container = screen.getByText('Thread').parentElement
    expect(container).toHaveStyle({ borderColor: '#9CA3AF' }) // gray for pending
  })

  it('renders thread with running status', () => {
    const runningThread = { ...baseThread, status: 'running' }
    render(<ThreadNode data={createNodeData(runningThread)} />)
    
    expect(screen.getByText('RUNNING')).toBeInTheDocument()
    const container = screen.getByText('Thread').parentElement
    expect(container).toHaveStyle({ borderColor: '#F59E0B' }) // yellow for running
  })

  it('renders thread with waiting status', () => {
    const waitingThread = { ...baseThread, status: 'waiting' }
    render(<ThreadNode data={createNodeData(waitingThread)} />)
    
    expect(screen.getByText('WAITING')).toBeInTheDocument()
    const container = screen.getByText('Thread').parentElement
    expect(container).toHaveStyle({ borderColor: '#3B82F6' }) // blue for waiting
  })

  it('renders thread with completed status', () => {
    const completedThread = { ...baseThread, status: 'completed' }
    render(<ThreadNode data={createNodeData(completedThread)} />)
    
    expect(screen.getByText('COMPLETED')).toBeInTheDocument()
    const container = screen.getByText('Thread').parentElement
    expect(container).toHaveStyle({ borderColor: '#10B981' }) // green for completed
  })

  it('renders thread with failed status', () => {
    const failedThread = { ...baseThread, status: 'failed' }
    render(<ThreadNode data={createNodeData(failedThread)} />)
    
    expect(screen.getByText('FAILED')).toBeInTheDocument()
    const container = screen.getByText('Thread').parentElement
    expect(container).toHaveStyle({ borderColor: '#EF4444' }) // red for failed
  })

  it('truncates goal text at 50 characters', () => {
    const longGoalThread = {
      ...baseThread,
      goal: 'This is a very long goal text that should be truncated after fifty characters'
    }
    render(<ThreadNode data={createNodeData(longGoalThread)} />)
    
    // The component truncates at 50 chars and adds "..."
    expect(screen.getByText('This is a very long goal text that should be trunc...')).toBeInTheDocument()
  })

  it('displays task progress for completed tasks', () => {
    const threadWithTasks = {
      ...baseThread,
      tasks: [
        { status: 'completed' },
        { status: 'completed' },
        { status: 'pending' }
      ] as any[]
    }
    render(<ThreadNode data={createNodeData(threadWithTasks)} />)
    
    expect(screen.getByText('Tasks: 2/3')).toBeInTheDocument()
  })

  it('does not display task progress when no tasks', () => {
    render(<ThreadNode data={createNodeData(baseThread)} />)
    
    expect(screen.queryByText(/Tasks:/)).not.toBeInTheDocument()
  })

  it('calls onClick when clicked', () => {
    render(<ThreadNode data={createNodeData(baseThread)} />)
    
    const container = screen.getByText('Thread').parentElement!
    fireEvent.click(container)
    
    expect(mockOnClick).toHaveBeenCalledTimes(1)
    expect(mockOnClick).toHaveBeenCalledWith(baseThread)
  })
})