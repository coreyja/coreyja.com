import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { ThreadGraphView } from './ThreadGraphView'
import { threadsApi } from '../api/threads'
import { renderWithQueryClient } from '../test-utils'

// Mock the API
vi.mock('../api/threads', () => ({
  threadsApi: {
    listThreads: vi.fn(() => Promise.resolve([])),
    getThread: vi.fn(),
  },
}))

// Mock React Flow with a simple implementation
vi.mock('@xyflow/react', async () => {
  const React = (await vi.importActual('react')) as typeof import('react')
  return {
    ReactFlow: ({ children }: { children?: React.ReactNode }) => {
      return React.createElement('div', { 'data-testid': 'react-flow' }, children)
    },
    useNodesState: () => [[], vi.fn(), vi.fn()],
    useEdgesState: () => [[], vi.fn(), vi.fn()],
    Controls: () => React.createElement('div', { 'data-testid': 'controls' }),
    Background: () => React.createElement('div', { 'data-testid': 'background' }),
    BackgroundVariant: { Dots: 'dots' },
  }
})

// Mock components
vi.mock('./ThreadNode', () => ({
  ThreadNode: () => null,
}))

vi.mock('./StitchNode', () => ({
  StitchNode: () => null,
}))

vi.mock('./ThreadDetailPanel', () => ({
  ThreadDetailPanel: () => null,
}))

describe('ThreadGraphView', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders loading state initially', () => {
    renderWithQueryClient(<ThreadGraphView />)
    expect(screen.getByText('Loading threads...')).toBeInTheDocument()
  })

  it('calls listThreads API on mount', async () => {
    renderWithQueryClient(<ThreadGraphView />)

    await waitFor(() => {
      expect(threadsApi.listThreads).toHaveBeenCalledTimes(1)
    })
  })

  it('renders without crashing when API returns empty array', async () => {
    vi.mocked(threadsApi.listThreads).mockResolvedValue([])

    const { container } = renderWithQueryClient(<ThreadGraphView />)

    await waitFor(() => {
      expect(container).toBeTruthy()
    })
  })

  it('renders without crashing when API fails', async () => {
    vi.mocked(threadsApi.listThreads).mockRejectedValue(new Error('API Error'))

    // Suppress console.error for this test
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {})

    const { container } = renderWithQueryClient(<ThreadGraphView />)

    await waitFor(() => {
      expect(container).toBeTruthy()
    })

    consoleError.mockRestore()
  })
})
