import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, fireEvent } from '@testing-library/react'
import { ThreadDetailPanel } from './ThreadDetailPanel'
import { Thread, Stitch } from '../types'
import { renderWithQueryClient } from '../test-utils'

describe('ThreadDetailPanel', () => {
  const mockOnClose = vi.fn()

  const baseThread: Thread = {
    thread_id: '123e4567-e89b-12d3-a456-426614174000',
    branching_stitch_id: null,
    goal: 'Test thread goal',
    tasks: [],
    status: 'completed',
    result: null,
    pending_child_results: [],
    thread_type: 'autonomous',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  }

  const baseStitch: Stitch = {
    stitch_id: '223e4567-e89b-12d3-a456-426614174000',
    thread_id: '123e4567-e89b-12d3-a456-426614174000',
    previous_stitch_id: null,
    stitch_type: 'tool_call',
    llm_request: undefined,
    llm_response: undefined,
    tool_name: 'database_query',
    tool_input: undefined,
    tool_output: undefined,
    child_thread_id: undefined,
    thread_result_summary: undefined,
    created_at: '2024-01-01T00:00:00Z',
  }

  beforeEach(() => {
    mockOnClose.mockClear()
  })

  it('renders nothing when no thread or stitch is provided', () => {
    const { container } = renderWithQueryClient(<ThreadDetailPanel onClose={mockOnClose} />)
    expect(container.firstChild).toBeNull()
  })

  it('renders thread details when thread is provided', () => {
    renderWithQueryClient(<ThreadDetailPanel thread={baseThread} onClose={mockOnClose} />)

    expect(screen.getByText('Thread Details')).toBeInTheDocument()
    expect(screen.getByText('123e4567-e89b-12d3-a456-426614174000')).toBeInTheDocument()
    expect(screen.getByText('Test thread goal')).toBeInTheDocument()
    expect(screen.getByText('completed')).toBeInTheDocument()
  })

  it('renders stitch details when stitch is provided', () => {
    renderWithQueryClient(<ThreadDetailPanel stitch={baseStitch} onClose={mockOnClose} />)

    expect(screen.getByText('Stitch Details')).toBeInTheDocument()
    expect(screen.getByText('223e4567-e89b-12d3-a456-426614174000')).toBeInTheDocument()
    expect(screen.getByText('tool_call')).toBeInTheDocument()
    expect(screen.getByText('database_query')).toBeInTheDocument()
  })

  it('calls onClose when close button is clicked', () => {
    renderWithQueryClient(<ThreadDetailPanel thread={baseThread} onClose={mockOnClose} />)

    const closeButton = screen.getByRole('button', { name: '×' })
    fireEvent.click(closeButton)

    expect(mockOnClose).toHaveBeenCalledTimes(1)
  })

  it('renders thread tasks when present', () => {
    const threadWithTasks = {
      ...baseThread,
      tasks: [
        { id: '1', status: 'completed' as const, description: 'Task 1' },
        { id: '2', status: 'in_progress' as const, description: 'Task 2' },
        { id: '3', status: 'pending' as const, description: 'Task 3' },
      ],
    }
    renderWithQueryClient(<ThreadDetailPanel thread={threadWithTasks} onClose={mockOnClose} />)

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
      result: { success: true, data: 'Result data' },
    }
    renderWithQueryClient(<ThreadDetailPanel thread={threadWithResult} onClose={mockOnClose} />)

    expect(screen.getByText('Result:')).toBeInTheDocument()
    expect(screen.getByText(/success/)).toBeInTheDocument()
    expect(screen.getByText(/Result data/)).toBeInTheDocument()
  })

  it('renders stitch tool details when present', () => {
    const stitchWithToolData = {
      ...baseStitch,
      tool_input: { query: 'SELECT * FROM users' },
      tool_output: { rows: 5, status: 'success' },
    }
    renderWithQueryClient(<ThreadDetailPanel stitch={stitchWithToolData} onClose={mockOnClose} />)

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
      llm_request: {
        model: 'test-model',
        messages: [{ role: 'user' as const, content: 'Test prompt' }],
      },
      llm_response: {
        id: 'test-id',
        model: 'test-model',
        choices: [
          {
            message: { role: 'assistant' as const, content: 'Test response' },
            finish_reason: 'stop',
          },
        ],
      },
    }
    renderWithQueryClient(<ThreadDetailPanel stitch={stitchWithLLM} onClose={mockOnClose} />)

    expect(screen.getByText('LLM Request:')).toBeInTheDocument()
    expect(screen.getByText(/Test prompt/)).toBeInTheDocument()
    expect(screen.getByText('LLM Response:')).toBeInTheDocument()
    expect(screen.getByText(/Test response/)).toBeInTheDocument()
  })

  it('does not render optional fields when not present', () => {
    renderWithQueryClient(<ThreadDetailPanel thread={baseThread} onClose={mockOnClose} />)

    expect(screen.queryByText('Tasks:')).not.toBeInTheDocument()
    expect(screen.queryByText('Result:')).not.toBeInTheDocument()
  })

  it('applies correct styling to the panel', () => {
    renderWithQueryClient(<ThreadDetailPanel thread={baseThread} onClose={mockOnClose} />)

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
      zIndex: '10',
    })
  })

  it('applies correct color coding to task statuses', () => {
    const threadWithTasks = {
      ...baseThread,
      tasks: [
        { id: '1', status: 'completed' as const, description: 'Completed task' },
        { id: '2', status: 'in_progress' as const, description: 'In progress task' },
        { id: '3', status: 'pending' as const, description: 'Pending task' },
      ],
    }
    renderWithQueryClient(<ThreadDetailPanel thread={threadWithTasks} onClose={mockOnClose} />)

    const completedStatus = screen.getByText('[completed]')
    expect(completedStatus).toHaveStyle({ color: 'rgb(0, 128, 0)' }) // green

    const inProgressStatus = screen.getByText('[in_progress]')
    expect(inProgressStatus).toHaveStyle({ color: 'rgb(255, 165, 0)' }) // orange

    const pendingStatus = screen.getByText('[pending]')
    expect(pendingStatus).toHaveStyle({ color: 'rgb(128, 128, 128)' }) // gray
  })
})
