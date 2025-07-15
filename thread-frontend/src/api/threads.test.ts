import { describe, it, expect, vi, beforeEach } from 'vitest'
import { Thread, ThreadWithStitches } from '../types'

// Use vi.hoisted to ensure mock setup happens before any imports
const { mockAxiosGet, mockAxiosPost, mockAxiosCreate } = vi.hoisted(() => {
  const get = vi.fn()
  const post = vi.fn()
  const create = vi.fn(() => ({
    get,
    post,
  }))
  return {
    mockAxiosGet: get,
    mockAxiosPost: post,
    mockAxiosCreate: create,
  }
})

vi.mock('axios', () => ({
  default: {
    create: mockAxiosCreate,
  }
}))

// Import after mocking
import { threadsApi } from './threads'

describe('threadsApi', () => {
  beforeEach(() => {
    // Clear all mocks before each test
    vi.clearAllMocks()
  })

  describe('listThreads', () => {
    it('fetches threads list successfully', async () => {
      const mockThreads: Thread[] = [
        {
          thread_id: '123',
          parent_thread_id: null,
          branching_stitch_id: null,
          goal: 'Test thread',
          tasks: [],
          status: 'running',
          result: null,
          pending_child_results: [],
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z'
        }
      ]

      mockAxiosGet.mockResolvedValue({
        data: { threads: mockThreads }
      })

      const result = await threadsApi.listThreads()

      expect(mockAxiosGet).toHaveBeenCalledWith('/threads')
      expect(result).toEqual(mockThreads)
    })

    it('handles API errors', async () => {
      const errorMessage = 'Network error'
      mockAxiosGet.mockRejectedValue(new Error(errorMessage))

      await expect(threadsApi.listThreads()).rejects.toThrow(errorMessage)
    })
  })

  describe('getThread', () => {
    it('fetches single thread with stitches successfully', async () => {
      const mockThreadWithStitches: ThreadWithStitches = {
        thread_id: '123',
        parent_thread_id: null,
        branching_stitch_id: null,
        goal: 'Test thread',
        tasks: [],
        status: 'completed',
        result: { success: true },
        pending_child_results: [],
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
        stitches: [
          {
            stitch_id: '456',
            thread_id: '123',
            previous_stitch_id: null,
            stitch_type: 'llm_call',
            llm_request: { prompt: 'test' },
            llm_response: { completion: 'response' },
            tool_name: null,
            tool_input: null,
            tool_output: null,
            child_thread_id: null,
            thread_result_summary: null,
            created_at: '2024-01-01T00:00:00Z'
          }
        ]
      }

      mockAxiosGet.mockResolvedValue({
        data: mockThreadWithStitches
      })

      const result = await threadsApi.getThread('123')

      expect(mockAxiosGet).toHaveBeenCalledWith('/threads/123')
      expect(result).toEqual(mockThreadWithStitches)
    })

    it('handles thread not found error', async () => {
      mockAxiosGet.mockRejectedValue({
        response: { status: 404, data: { error: 'Thread not found' } }
      })

      await expect(threadsApi.getThread('999')).rejects.toMatchObject({
        response: { status: 404 }
      })
    })
  })

  describe('createThread', () => {
    it('creates thread successfully', async () => {
      const newThread: Thread = {
        thread_id: '789',
        parent_thread_id: null,
        branching_stitch_id: null,
        goal: 'New thread goal',
        tasks: [],
        status: 'pending',
        result: null,
        pending_child_results: [],
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z'
      }

      mockAxiosPost.mockResolvedValue({
        data: newThread
      })

      const result = await threadsApi.createThread('New thread goal')

      expect(mockAxiosPost).toHaveBeenCalledWith('/threads', { goal: 'New thread goal' })
      expect(result).toEqual(newThread)
    })

    it('handles validation error for empty goal', async () => {
      mockAxiosPost.mockRejectedValue({
        response: { status: 400, data: { error: 'Goal is required' } }
      })

      await expect(threadsApi.createThread('')).rejects.toMatchObject({
        response: { status: 400 }
      })
    })
  })
})