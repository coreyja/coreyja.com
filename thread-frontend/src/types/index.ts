export interface Thread {
  id: string;
  parent_thread_id: string | null;
  branching_stitch_id: string | null;
  goal: string;
  tasks: any[];
  status: 'pending' | 'running' | 'waiting' | 'completed' | 'failed';
  result: any | null;
  pending_child_results: any[];
  created_at: string;
  updated_at: string;
}

export interface Stitch {
  id: string;
  thread_id: string;
  previous_stitch_id: string | null;
  stitch_type: 'llm_call' | 'tool_call' | 'thread_result';
  // LLM call fields
  llm_request?: any;
  llm_response?: any;
  // Tool call fields
  tool_name?: string;
  tool_input?: any;
  tool_output?: any;
  // Thread result fields
  child_thread_id?: string;
  thread_result_summary?: string;
  created_at: string;
}

export interface ThreadWithStitches extends Thread {
  stitches: Stitch[];
}

export interface ThreadsListResponse {
  threads: Thread[];
}