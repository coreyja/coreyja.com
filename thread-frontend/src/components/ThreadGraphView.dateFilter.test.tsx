import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { RouterProvider, createMemoryHistory, createRouter } from '@tanstack/react-router'
import { routeTree } from '../routeTree.gen'
import * as api from '../api/threads'

// Mock the API module
vi.mock('../api/threads', () => ({
  threadsApi: {
    listThreads: vi.fn(),
    getThread: vi.fn(),
    getThreadChildren: vi.fn(),
    getThreadParents: vi.fn(),
  },
}))

// Mock ReactFlow to avoid rendering issues in tests
vi.mock('@xyflow/react', () => ({
  ReactFlow: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="react-flow">{children}</div>
  ),
  Controls: () => <div data-testid="controls" />,
  Background: () => <div data-testid="background" />,
  BackgroundVariant: { Dots: 'dots' },
}))

describe('ThreadGraphView date filter integration', () => {
  let queryClient: QueryClient
  let router: ReturnType<typeof createRouter>

  beforeEach(() => {
    vi.clearAllMocks()
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          refetchInterval: false,
        },
      },
    })

    const history = createMemoryHistory({
      initialEntries: ['/'],
    })

    router = createRouter({
      routeTree,
      history,
    })
  })

  it('passes days parameter to API when changed', async () => {
    const mockThreads = [
      {
        thread_id: 'test-thread-1',
        branching_stitch_id: null,
        goal: 'Test thread 1',
        tasks: [],
        status: 'completed' as const,
        result: { success: true },
        pending_child_results: [],
        thread_type: 'autonomous' as const,
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
        stitch_count: 2,
        children_count: 0,
      },
    ]

    vi.mocked(api.threadsApi.listThreads).mockResolvedValue(mockThreads)

    render(
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    )

    // Wait for initial render with default 3 days
    await waitFor(() => {
      expect(api.threadsApi.listThreads).toHaveBeenCalledWith(3)
    })

    // Find and change the date filter
    const select = screen.getByRole('combobox')
    const user = userEvent.setup()

    await user.selectOptions(select, '7')

    // Verify API was called with 7 days
    await waitFor(() => {
      expect(api.threadsApi.listThreads).toHaveBeenCalledWith(7)
    })
  })

  it('preserves days parameter when navigating between threads', async () => {
    const mockThreads = [
      {
        thread_id: 'test-thread-1',
        branching_stitch_id: null,
        goal: 'Test thread 1',
        tasks: [],
        status: 'completed' as const,
        result: { success: true },
        pending_child_results: [],
        thread_type: 'autonomous' as const,
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
        stitch_count: 2,
        children_count: 0,
      },
    ]

    vi.mocked(api.threadsApi.listThreads).mockResolvedValue(mockThreads)

    // Start with days=7 in the URL
    const history = createMemoryHistory({
      initialEntries: ['/?days=7'],
    })

    router = createRouter({
      routeTree,
      history,
    })

    render(
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    )

    // Verify initial API call with 7 days
    await waitFor(() => {
      expect(api.threadsApi.listThreads).toHaveBeenCalledWith(7)
    })

    // Verify the select shows 7 days
    const select = screen.getByRole('combobox')
    expect(select).toHaveValue('7')
  })
})
