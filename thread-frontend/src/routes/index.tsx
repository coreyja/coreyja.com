// src/routes/index.tsx
import { createFileRoute } from '@tanstack/react-router'
import { ThreadGraphView } from '../components/ThreadGraphView'
import { z } from 'zod'

const threadSearchSchema = z.object({
  thread: z.string().optional(),
  stitch: z.string().optional(),
})

export const Route = createFileRoute('/')({
  validateSearch: threadSearchSchema,
  component: Home,
})

function Home() {
  const { thread, stitch } = Route.useSearch()
  return <ThreadGraphView threadId={thread} stitchId={stitch} />
}
