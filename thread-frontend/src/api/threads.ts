import axios from 'axios'
import {
  Thread,
  ThreadWithStitches,
  ThreadWithCounts,
  ThreadsWithCountsResponse,
  ChildrenResponse,
  Message,
} from '../types'

const API_BASE_URL = '/admin/api'

const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
})

export const threadsApi = {
  listThreads: async (daysBack?: number): Promise<ThreadWithCounts[]> => {
    const response = await api.get<ThreadsWithCountsResponse>('/threads', {
      params: daysBack !== undefined ? { days_back: daysBack } : undefined,
    })
    return response.data.threads
  },

  listRecentThreads: async (daysBack: number = 3): Promise<ThreadWithCounts[]> => {
    const response = await api.get<ThreadsWithCountsResponse>('/threads', {
      params: { days_back: daysBack },
    })
    return response.data.threads
  },

  getThread: async (id: string): Promise<ThreadWithStitches> => {
    const response = await api.get<ThreadWithStitches>(`/threads/${id}`)
    return response.data
  },

  getThreadChildren: async (id: string): Promise<ThreadWithCounts[]> => {
    const response = await api.get<ChildrenResponse>(`/threads/${id}/children`)
    return response.data.children
  },

  getThreadParents: async (id: string): Promise<Thread[]> => {
    const response = await api.get<Thread[]>(`/threads/${id}/parents`)
    return response.data
  },

  createThread: async (goal: string): Promise<Thread> => {
    const response = await api.post<Thread>('/threads', { goal })
    return response.data
  },

  getThreadMessages: async (id: string): Promise<Message[]> => {
    const response = await api.get<Message[]>(`/threads/${id}/messages`)
    return response.data
  },
}
