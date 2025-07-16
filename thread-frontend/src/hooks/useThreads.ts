import { useQuery } from '@tanstack/react-query'
import { threadsApi } from '../api/threads'

export const THREADS_QUERY_KEY = ['threads'] as const
export const THREAD_QUERY_KEY = (id: string) => ['thread', id] as const
export const ALL_THREAD_DETAILS_QUERY_KEY = (threadIds: string[]) =>
  ['all-thread-details', threadIds] as const

export const useThreads = () => {
  return useQuery({
    queryKey: THREADS_QUERY_KEY,
    queryFn: threadsApi.listThreads,
    refetchInterval: 2000, // Auto-refresh every 2 seconds to match current behavior
  })
}

export const useThread = (id: string | undefined) => {
  return useQuery({
    queryKey: THREAD_QUERY_KEY(id!),
    queryFn: () => threadsApi.getThread(id!),
    enabled: !!id,
  })
}

export const useAllThreadDetails = (threadIds: string[] | undefined) => {
  return useQuery({
    queryKey: ALL_THREAD_DETAILS_QUERY_KEY(threadIds ?? []),
    queryFn: async () => {
      if (!threadIds || threadIds.length === 0) return []
      return Promise.all(threadIds.map(id => threadsApi.getThread(id)))
    },
    enabled: !!threadIds && threadIds.length > 0,
  })
}
