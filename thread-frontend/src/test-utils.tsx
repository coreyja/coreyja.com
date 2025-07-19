import React from 'react'
import { render } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { createMemoryHistory, createRouter, RouterProvider } from '@tanstack/react-router'
import { routeTree } from './routeTree.gen'

export function createTestQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
      },
    },
  })
}

export function renderWithQueryClient(ui: React.ReactElement) {
  const testQueryClient = createTestQueryClient()
  return render(<QueryClientProvider client={testQueryClient}>{ui}</QueryClientProvider>)
}

export function renderWithProviders(ui: React.ReactElement) {
  const testQueryClient = createTestQueryClient()

  // Create a memory history and router for testing
  const memoryHistory = createMemoryHistory({
    initialEntries: ['/'],
  })

  const router = createRouter({
    routeTree,
    history: memoryHistory,
    defaultComponent: () => ui,
  })

  return render(
    <QueryClientProvider client={testQueryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  )
}
