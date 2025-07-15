import React from 'react'
import ReactDOM from 'react-dom/client'
import { ThreadGraphView } from './components/ThreadGraphView'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ThreadGraphView />
  </React.StrictMode>,
)