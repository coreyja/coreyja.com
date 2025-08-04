// src/routes/index.tsx
import { createFileRoute } from '@tanstack/react-router'
import { ThreadGraphView } from '../components/ThreadGraphView'
import { z } from 'zod'

const threadSearchSchema = z.object({
  thread: z.string().optional(),
  stitch: z.string().optional(),
  days: z.coerce.number().int().min(1).max(7).optional().default(3),
})

export const Route = createFileRoute('/')({
  validateSearch: threadSearchSchema,
  component: Home,
})

function Home() {
  const { thread, stitch, days } = Route.useSearch()
  return <ThreadGraphView threadId={thread} stitchId={stitch} daysBack={days} />
}
