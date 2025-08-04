import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { ThreadGraphView } from './ThreadGraphView'
import { threadsApi } from '../api/threads'
import { renderWithProviders } from '../test-utils'

// Mock the API
vi.mock('../api/threads', () => ({
  threadsApi: {
    listThreads: vi.fn(() => new Promise(() => {})), // Never resolves to simulate loading
    getThread: vi.fn(),
    getThreadChildren: vi.fn(() => Promise.resolve([])),
    getThreadParents: vi.fn(() => Promise.resolve([])),
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
    // Reset mocks to default behavior (loading state)
    vi.mocked(threadsApi.listThreads).mockImplementation(() => new Promise(() => {}))
  })

  it('renders loading state initially', async () => {
    // Suppress console logs for this test
    const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => {})

    renderWithProviders(<ThreadGraphView />)

    // Check for loading text
    await waitFor(() => {
      expect(screen.getByText('Loading threads...')).toBeInTheDocument()
    })

    consoleSpy.mockRestore()
  })

  it('calls listThreads API on mount', async () => {
    // Reset the mock to resolve properly for this test
    vi.mocked(threadsApi.listThreads).mockResolvedValue([])

    renderWithProviders(<ThreadGraphView />)

    await waitFor(() => {
      expect(threadsApi.listThreads).toHaveBeenCalledTimes(1)
    })
  })

  it('renders without crashing when API returns empty array', async () => {
    vi.mocked(threadsApi.listThreads).mockResolvedValue([])

    const { container } = renderWithProviders(<ThreadGraphView />)

    await waitFor(() => {
      expect(container).toBeTruthy()
    })
  })

  it('renders without crashing when API fails', async () => {
    vi.mocked(threadsApi.listThreads).mockRejectedValue(new Error('API Error'))

    // Suppress console.error for this test
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {})

    const { container } = renderWithProviders(<ThreadGraphView />)

    await waitFor(() => {
      expect(container).toBeTruthy()
    })

    consoleError.mockRestore()
  })
})
