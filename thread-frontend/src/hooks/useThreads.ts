import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { threadsApi } from '../api/threads'

export const THREADS_QUERY_KEY = ['threads'] as const
export const THREAD_QUERY_KEY = (id: string) => ['thread', id] as const

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

export const useCreateThread = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (goal: string) => threadsApi.createThread(goal),
    onSuccess: () => {
      // Invalidate and refetch threads list
      queryClient.invalidateQueries({ queryKey: THREADS_QUERY_KEY })
    },
  })
}
