import { ReactElement } from 'react'
import { render, RenderOptions } from '@testing-library/react'
import { RouterProvider, createRouter } from '@tanstack/react-router'
import { routeTree } from '../routeTree.gen'

// Create a test router for component tests
export function createTestRouter() {
  return createRouter({
    routeTree,
    defaultPreload: false,
  })
}

// Custom render function that includes providers
export const renderWithProviders = (ui: ReactElement, options?: Omit<RenderOptions, 'wrapper'>) => {
  const router = createTestRouter()

  return render(<RouterProvider router={router} />, options)
}

// eslint-disable-next-line react-refresh/only-export-components
export * from '@testing-library/react'
export { renderWithProviders as render }
