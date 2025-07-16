// src/routes/index.tsx
import { createFileRoute } from '@tanstack/react-router'
import { ThreadGraphView } from '../components/ThreadGraphView'

export const Route = createFileRoute('/')({
  component: Home,
})

function Home() {
  return <ThreadGraphView />
}
