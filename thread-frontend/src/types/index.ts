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

// Note: These types are simplified and don't match the actual Anthropic API format
// The frontend displays raw JSON so it works with any structure
export interface LLMRequest {
  [key: string]: unknown
}

export interface LLMResponse {
  [key: string]: unknown
}

export type ToolInput = Record<string, unknown>
export type ToolOutput = Record<string, unknown>

export interface DiscordMetadata {
  thread_id: string
  discord_thread_id: string
  channel_id: string
  guild_id: string
  last_message_id?: string
  created_by: string
  thread_name: string
  participants: string[]
  webhook_url?: string
  created_at: string
  updated_at: string
}

export interface Thread {
  thread_id: string
  branching_stitch_id: string | null
  goal: string
  tasks: Task[]
  status: 'pending' | 'running' | 'waiting' | 'completed' | 'failed' | 'aborted'
  result: ThreadResult | null
  pending_child_results: ChildResult[]
  thread_type: 'autonomous' | 'interactive'
  discord_metadata?: DiscordMetadata
  created_at: string
  updated_at: string
}

export interface Stitch {
  stitch_id: string
  thread_id: string
  previous_stitch_id: string | null
  stitch_type: 'initial_prompt' | 'llm_call' | 'tool_call' | 'thread_result' | 'discord_message'
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
  children: ThreadWithCounts[]
}
