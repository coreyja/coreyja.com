export interface Task {
  id: string
  description: string
  status: 'pending' | 'in_progress' | 'completed'
}

export interface ThreadResult {
  success: boolean
  data?: unknown
  error?: string
}

export interface ChildResult {
  thread_id: string
  result: ThreadResult
}

export interface LLMRequest {
  model: string
  messages: Array<{
    role: 'system' | 'user' | 'assistant'
    content: string
  }>
  temperature?: number
  max_tokens?: number
}

export interface LLMResponse {
  id: string
  model: string
  choices: Array<{
    message: {
      role: 'assistant'
      content: string
    }
    finish_reason: string
  }>
  usage?: {
    prompt_tokens: number
    completion_tokens: number
    total_tokens: number
  }
}

export type ToolInput = Record<string, unknown>
export type ToolOutput = Record<string, unknown>

export interface Thread {
  thread_id: string
  branching_stitch_id: string | null
  goal: string
  tasks: Task[]
  status: 'pending' | 'running' | 'waiting' | 'completed' | 'failed' | 'aborted'
  result: ThreadResult | null
  pending_child_results: ChildResult[]
  created_at: string
  updated_at: string
}

export interface Stitch {
  stitch_id: string
  thread_id: string
  previous_stitch_id: string | null
  stitch_type: 'llm_call' | 'tool_call' | 'thread_result'
  llm_request?: LLMRequest
  llm_response?: LLMResponse
  tool_name?: string
  tool_input?: ToolInput
  tool_output?: ToolOutput
  child_thread_id?: string
  thread_result_summary?: string
  created_at: string
}

export interface ThreadWithStitches extends Thread {
  stitches: Stitch[]
}

export interface ThreadsListResponse {
  threads: Thread[]
}

export interface ThreadWithCounts extends Thread {
  stitch_count: number
  children_count: number
}

export interface ThreadsWithCountsResponse {
  threads: ThreadWithCounts[]
}

export interface ChildrenResponse {
  children: Thread[]
}
