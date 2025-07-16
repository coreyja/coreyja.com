import { useQuery } from '@tanstack/react-query'
import { threadsApi } from '../api/threads'

export const THREADS_QUERY_KEY = ['threads'] as const
export const RECENT_THREADS_QUERY_KEY = ['threads', 'recent'] as const
export const THREAD_QUERY_KEY = (id: string) => ['thread', id] as const
export const THREAD_CHILDREN_QUERY_KEY = (id: string) => ['thread', id, 'children'] as const
export const THREAD_PARENTS_QUERY_KEY = (id: string) => ['thread', id, 'parents'] as const
export const ALL_THREAD_DETAILS_QUERY_KEY = (threadIds: string[]) =>
  ['all-thread-details', threadIds] as const

export const useThreads = () => {
  return useQuery({
    queryKey: THREADS_QUERY_KEY,
    queryFn: threadsApi.listThreads,
    refetchInterval: 2000, // Auto-refresh every 2 seconds to match current behavior
  })
}

export const useRecentThreads = () => {
  return useQuery({
    queryKey: RECENT_THREADS_QUERY_KEY,
    queryFn: threadsApi.listRecentThreads,
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

export const useThreadChildren = (id: string | undefined) => {
  return useQuery({
    queryKey: THREAD_CHILDREN_QUERY_KEY(id!),
    queryFn: () => threadsApi.getThreadChildren(id!),
    enabled: !!id,
  })
}

export const useThreadParents = (id: string | undefined) => {
  return useQuery({
    queryKey: THREAD_PARENTS_QUERY_KEY(id!),
    queryFn: () => threadsApi.getThreadParents(id!),
    enabled: !!id,
  })
}
