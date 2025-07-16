/* eslint-disable @typescript-eslint/no-explicit-any */
import { Client } from 'pg'
import * as dotenv from 'dotenv'

// Load environment variables
dotenv.config()

// Database connection configuration
const client = new Client({
  connectionString: process.env.DATABASE_URL || 'postgresql://localhost/coreyja_development',
})

interface ThreadData {
  thread_id: string
  branching_stitch_id: string | null
  goal: string
  tasks: any[]
  status: string
  result: any | null
  pending_child_results: any[]
}

interface StitchData {
  stitch_id: string
  thread_id: string
  previous_stitch_id: string | null
  stitch_type: string
  llm_request?: any
  llm_response?: any
  tool_name?: string
  tool_input?: any
  tool_output?: any
  child_thread_id?: string
  thread_result_summary?: string
}

async function seedDatabase() {
  try {
    await client.connect()
    console.log('Connected to database')

    // Start transaction
    await client.query('BEGIN')

    // Clean up existing data
    console.log('Cleaning up existing data...')
    await client.query('TRUNCATE TABLE stitches, threads RESTART IDENTITY CASCADE')

    // Define all threads first
    const threads: ThreadData[] = [
      {
        thread_id: 'a1b2c3d4-e5f6-7890-abcd-ef1234567801',
        branching_stitch_id: null,
        goal: 'Sample: Review and improve code quality',
        tasks: [
          { id: 'task-1', description: 'Analyze code structure', status: 'completed' },
          { id: 'task-2', description: 'Check for security issues', status: 'completed' },
          { id: 'task-3', description: 'Suggest improvements', status: 'in_progress' },
        ],
        status: 'running',
        result: null,
        pending_child_results: [],
      },
      {
        thread_id: 'a2b2c3d4-e5f6-7890-abcd-ef1234567802',
        branching_stitch_id: 'b3b2c3d4-e5f6-7890-abcd-ef1234567803',
        goal: 'Sample: Perform security analysis',
        tasks: [
          { id: 'task-1', description: 'Scan for vulnerabilities', status: 'completed' },
          { id: 'task-2', description: 'Check dependencies', status: 'completed' },
        ],
        status: 'completed',
        result: { success: true, data: { vulnerabilities: 2, severity: 'medium' } },
        pending_child_results: [],
      },
      {
        thread_id: 'a3b2c3d4-e5f6-7890-abcd-ef1234567803',
        branching_stitch_id: null,
        goal: 'Sample: Process and analyze user data',
        tasks: [
          { id: 'task-1', description: 'Load data from database', status: 'completed' },
          { id: 'task-2', description: 'Transform data', status: 'completed' },
          { id: 'task-3', description: 'Generate reports', status: 'running' },
        ],
        status: 'running',
        result: null,
        pending_child_results: [
          { thread_id: 'a4b2c3d4-e5f6-7890-abcd-ef1234567804', result: null },
          { thread_id: 'a5b2c3d4-e5f6-7890-abcd-ef1234567805', result: null },
        ],
      },
      {
        thread_id: 'a4b2c3d4-e5f6-7890-abcd-ef1234567804',
        branching_stitch_id: 'b7b2c3d4-e5f6-7890-abcd-ef1234567807',
        goal: 'Sample: Transform raw data',
        tasks: [
          { id: 'task-1', description: 'Clean data', status: 'completed' },
          { id: 'task-2', description: 'Normalize values', status: 'completed' },
        ],
        status: 'completed',
        result: { success: true, data: { records_processed: 1523 } },
        pending_child_results: [],
      },
      {
        thread_id: 'a5b2c3d4-e5f6-7890-abcd-ef1234567805',
        branching_stitch_id: 'b8b2c3d4-e5f6-7890-abcd-ef1234567808',
        goal: 'Sample: Generate analytics reports',
        tasks: [
          { id: 'task-1', description: 'Create visualizations', status: 'completed' },
          { id: 'task-2', description: 'Generate summary', status: 'running' },
        ],
        status: 'running',
        result: null,
        pending_child_results: [
          { thread_id: 'a6b2c3d4-e5f6-7890-abcd-ef1234567806', result: null },
        ],
      },
      {
        thread_id: 'a6b2c3d4-e5f6-7890-abcd-ef1234567806',
        branching_stitch_id: 'bab2c3d4-e5f6-7890-abcd-ef1234567810',
        goal: 'Sample: Export report to PDF, Excel, and email',
        tasks: [
          { id: 'task-1', description: 'Generate PDF', status: 'completed' },
          { id: 'task-2', description: 'Generate Excel', status: 'completed' },
          { id: 'task-3', description: 'Send email', status: 'pending' },
        ],
        status: 'waiting',
        result: null,
        pending_child_results: [],
      },
      {
        thread_id: 'a7b2c3d4-e5f6-7890-abcd-ef1234567807',
        branching_stitch_id: null,
        goal: 'Sample: Failed task - API integration',
        tasks: [
          { id: 'task-1', description: 'Connect to external API', status: 'completed' },
          { id: 'task-2', description: 'Fetch data', status: 'in_progress' },
        ],
        status: 'failed',
        result: { success: false, error: 'API rate limit exceeded' },
        pending_child_results: [],
      },
      {
        thread_id: 'a8b2c3d4-e5f6-7890-abcd-ef1234567808',
        branching_stitch_id: 'b5a2c3d4-e5f6-7890-abcd-ef1234567805',
        goal: 'Sample: Fix identified security vulnerabilities',
        tasks: [
          { id: 'task-1', description: 'Patch SQL injection vulnerability', status: 'completed' },
          { id: 'task-2', description: 'Fix XSS vulnerability', status: 'completed' },
          { id: 'task-3', description: 'Update security tests', status: 'pending' },
        ],
        status: 'running',
        result: null,
        pending_child_results: [],
      },
      {
        thread_id: 'a9b2c3d4-e5f6-7890-abcd-ef1234567809',
        branching_stitch_id: 'b3c2c3d4-e5f6-7890-abcd-ef1234567804',
        goal: 'Sample: Analyze code performance and optimization opportunities',
        tasks: [
          { id: 'task-1', description: 'Profile code execution', status: 'completed' },
          { id: 'task-2', description: 'Identify bottlenecks', status: 'completed' },
          { id: 'task-3', description: 'Suggest optimizations', status: 'completed' },
        ],
        status: 'completed',
        result: { success: true, data: { bottlenecks: 3, potential_speedup: '40%' } },
        pending_child_results: [],
      },
    ]

    // Insert threads (initially with null branching_stitch_id)
    console.log('Inserting threads...')
    for (const thread of threads) {
      await client.query(
        `INSERT INTO threads (thread_id, branching_stitch_id, goal, tasks, status, result, pending_child_results, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, NOW() - INTERVAL '2 hours', NOW())`,
        [
          thread.thread_id,
          null, // We'll update this later
          thread.goal,
          JSON.stringify(thread.tasks),
          thread.status,
          thread.result ? JSON.stringify(thread.result) : null,
          JSON.stringify(thread.pending_child_results),
        ]
      )
    }
    console.log(`Inserted ${threads.length} threads`)

    // Define all stitches
    const stitches: StitchData[] = [
      // Thread 1 stitches
      {
        stitch_id: 'b1b2c3d4-e5f6-7890-abcd-ef1234567801',
        thread_id: 'a1b2c3d4-e5f6-7890-abcd-ef1234567801',
        previous_stitch_id: null,
        stitch_type: 'llm_call',
        llm_request: {
          model: 'gpt-4',
          messages: [{ role: 'user', content: 'Analyze this code structure' }],
        },
        llm_response: {
          choices: [{ message: { content: 'I will analyze the code structure...' } }],
        },
      },
      {
        stitch_id: 'b2b2c3d4-e5f6-7890-abcd-ef1234567802',
        thread_id: 'a1b2c3d4-e5f6-7890-abcd-ef1234567801',
        previous_stitch_id: 'b1b2c3d4-e5f6-7890-abcd-ef1234567801',
        stitch_type: 'tool_call',
        tool_name: 'code_analyzer',
        tool_input: { file: 'main.py', analysis_type: 'structure' },
        tool_output: { complexity: 7, lines: 234, functions: 12 },
      },
      {
        stitch_id: 'b3b2c3d4-e5f6-7890-abcd-ef1234567803',
        thread_id: 'a1b2c3d4-e5f6-7890-abcd-ef1234567801',
        previous_stitch_id: 'b2b2c3d4-e5f6-7890-abcd-ef1234567802',
        stitch_type: 'thread_result',
        child_thread_id: 'a2b2c3d4-e5f6-7890-abcd-ef1234567802',
        thread_result_summary: 'Security scan completed: 2 medium issues found',
      },
      {
        stitch_id: 'b3c2c3d4-e5f6-7890-abcd-ef1234567804',
        thread_id: 'a1b2c3d4-e5f6-7890-abcd-ef1234567801',
        previous_stitch_id: 'b3b2c3d4-e5f6-7890-abcd-ef1234567803',
        stitch_type: 'thread_result',
        child_thread_id: 'a9b2c3d4-e5f6-7890-abcd-ef1234567809',
        thread_result_summary: 'Performance analysis initiated',
      },
      {
        stitch_id: 'b3d2c3d4-e5f6-7890-abcd-ef1234567805',
        thread_id: 'a1b2c3d4-e5f6-7890-abcd-ef1234567801',
        previous_stitch_id: 'b3c2c3d4-e5f6-7890-abcd-ef1234567804',
        stitch_type: 'llm_call',
        llm_request: {
          model: 'gpt-4',
          messages: [{ role: 'user', content: 'Compile final code review report' }],
        },
        llm_response: {
          choices: [
            {
              message: {
                content: 'Code review complete. Security and performance analyses delegated...',
              },
            },
          ],
        },
      },
      {
        stitch_id: 'b3e2c3d4-e5f6-7890-abcd-ef1234567806',
        thread_id: 'a1b2c3d4-e5f6-7890-abcd-ef1234567801',
        previous_stitch_id: 'b3d2c3d4-e5f6-7890-abcd-ef1234567805',
        stitch_type: 'tool_call',
        tool_name: 'report_generator',
        tool_input: { format: 'markdown', include_children: true },
        tool_output: { report_url: '/reports/code-review-123.md', size: '4.2KB' },
      },
      // Thread 2 stitches
      {
        stitch_id: 'b4b2c3d4-e5f6-7890-abcd-ef1234567804',
        thread_id: 'a2b2c3d4-e5f6-7890-abcd-ef1234567802',
        previous_stitch_id: null,
        stitch_type: 'tool_call',
        tool_name: 'security_scanner',
        tool_input: { scan_type: 'full', include_deps: true },
        tool_output: {
          vulnerabilities: [
            { type: 'SQL_INJECTION', severity: 'medium' },
            { type: 'XSS', severity: 'medium' },
          ],
        },
      },
      {
        stitch_id: 'b5b2c3d4-e5f6-7890-abcd-ef1234567805',
        thread_id: 'a2b2c3d4-e5f6-7890-abcd-ef1234567802',
        previous_stitch_id: 'b4b2c3d4-e5f6-7890-abcd-ef1234567804',
        stitch_type: 'llm_call',
        llm_request: {
          model: 'gpt-4',
          messages: [{ role: 'user', content: 'Summarize security findings' }],
        },
        llm_response: { choices: [{ message: { content: 'Found 2 medium severity issues...' } }] },
      },
      {
        stitch_id: 'b5a2c3d4-e5f6-7890-abcd-ef1234567805',
        thread_id: 'a2b2c3d4-e5f6-7890-abcd-ef1234567802',
        previous_stitch_id: 'b5b2c3d4-e5f6-7890-abcd-ef1234567805',
        stitch_type: 'thread_result',
        child_thread_id: 'a8b2c3d4-e5f6-7890-abcd-ef1234567808',
        thread_result_summary: 'Fixing security vulnerabilities',
      },
      // Thread 3 stitches
      {
        stitch_id: 'b6b2c3d4-e5f6-7890-abcd-ef1234567806',
        thread_id: 'a3b2c3d4-e5f6-7890-abcd-ef1234567803',
        previous_stitch_id: null,
        stitch_type: 'tool_call',
        tool_name: 'database_query',
        tool_input: { query: "SELECT * FROM users WHERE created_at > NOW() - INTERVAL '7 days'" },
        tool_output: { row_count: 1523, execution_time: '45ms' },
      },
      {
        stitch_id: 'b7b2c3d4-e5f6-7890-abcd-ef1234567807',
        thread_id: 'a3b2c3d4-e5f6-7890-abcd-ef1234567803',
        previous_stitch_id: 'b6b2c3d4-e5f6-7890-abcd-ef1234567806',
        stitch_type: 'thread_result',
        child_thread_id: 'a4b2c3d4-e5f6-7890-abcd-ef1234567804',
        thread_result_summary: 'Data transformation completed',
      },
      {
        stitch_id: 'b8b2c3d4-e5f6-7890-abcd-ef1234567808',
        thread_id: 'a3b2c3d4-e5f6-7890-abcd-ef1234567803',
        previous_stitch_id: 'b7b2c3d4-e5f6-7890-abcd-ef1234567807',
        stitch_type: 'thread_result',
        child_thread_id: 'a5b2c3d4-e5f6-7890-abcd-ef1234567805',
        thread_result_summary: 'Report generation started',
      },
      // Thread 5 stitches
      {
        stitch_id: 'b9b2c3d4-e5f6-7890-abcd-ef1234567808',
        thread_id: 'a5b2c3d4-e5f6-7890-abcd-ef1234567805',
        previous_stitch_id: null,
        stitch_type: 'tool_call',
        tool_name: 'report_builder',
        tool_input: { type: 'analytics', format: 'interactive' },
        tool_output: { charts: 5, tables: 3 },
      },
      {
        stitch_id: 'bab2c3d4-e5f6-7890-abcd-ef1234567810',
        thread_id: 'a5b2c3d4-e5f6-7890-abcd-ef1234567805',
        previous_stitch_id: 'b9b2c3d4-e5f6-7890-abcd-ef1234567808',
        stitch_type: 'thread_result',
        child_thread_id: 'a6b2c3d4-e5f6-7890-abcd-ef1234567806',
        thread_result_summary: 'Exporting report to multiple formats',
      },
      // Thread 8 stitches (grandchild)
      {
        stitch_id: 'b8a2c3d4-e5f6-7890-abcd-ef1234567808',
        thread_id: 'a8b2c3d4-e5f6-7890-abcd-ef1234567808',
        previous_stitch_id: null,
        stitch_type: 'tool_call',
        tool_name: 'code_patcher',
        tool_input: { vulnerability: 'SQL_INJECTION', file: 'database.py' },
        tool_output: { status: 'patched', lines_changed: 15 },
      },
      {
        stitch_id: 'b8b2c3d4-e5f6-7890-abcd-ef1234567809',
        thread_id: 'a8b2c3d4-e5f6-7890-abcd-ef1234567808',
        previous_stitch_id: 'b8a2c3d4-e5f6-7890-abcd-ef1234567808',
        stitch_type: 'tool_call',
        tool_name: 'code_patcher',
        tool_input: { vulnerability: 'XSS', file: 'views.py' },
        tool_output: { status: 'patched', lines_changed: 8 },
      },
      // Thread 9 stitches
      {
        stitch_id: 'b9a2c3d4-e5f6-7890-abcd-ef1234567809',
        thread_id: 'a9b2c3d4-e5f6-7890-abcd-ef1234567809',
        previous_stitch_id: null,
        stitch_type: 'tool_call',
        tool_name: 'profiler',
        tool_input: { mode: 'cpu', duration: 60 },
        tool_output: { hot_functions: ['process_data', 'calculate_metrics', 'render_output'] },
      },
      {
        stitch_id: 'b9b2c3d4-e5f6-7890-abcd-ef1234567810',
        thread_id: 'a9b2c3d4-e5f6-7890-abcd-ef1234567809',
        previous_stitch_id: 'b9a2c3d4-e5f6-7890-abcd-ef1234567809',
        stitch_type: 'llm_call',
        llm_request: {
          model: 'gpt-4',
          messages: [{ role: 'user', content: 'Analyze performance bottlenecks' }],
        },
        llm_response: {
          choices: [{ message: { content: 'The main bottlenecks are in data processing...' } }],
        },
      },
    ]

    // Insert stitches
    console.log('Inserting stitches...')
    for (const stitch of stitches) {
      await client.query(
        `INSERT INTO stitches (stitch_id, thread_id, previous_stitch_id, stitch_type, llm_request, llm_response, tool_name, tool_input, tool_output, child_thread_id, thread_result_summary, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW() - INTERVAL '1 hour')`,
        [
          stitch.stitch_id,
          stitch.thread_id,
          stitch.previous_stitch_id,
          stitch.stitch_type,
          stitch.llm_request ? JSON.stringify(stitch.llm_request) : null,
          stitch.llm_response ? JSON.stringify(stitch.llm_response) : null,
          stitch.tool_name || null,
          stitch.tool_input ? JSON.stringify(stitch.tool_input) : null,
          stitch.tool_output ? JSON.stringify(stitch.tool_output) : null,
          stitch.child_thread_id || null,
          stitch.thread_result_summary || null,
        ]
      )
    }
    console.log(`Inserted ${stitches.length} stitches`)

    // Now update threads with their branching_stitch_id
    console.log('Updating thread branching relationships...')
    for (const thread of threads) {
      if (thread.branching_stitch_id) {
        await client.query(`UPDATE threads SET branching_stitch_id = $1 WHERE thread_id = $2`, [
          thread.branching_stitch_id,
          thread.thread_id,
        ])
      }
    }
    console.log('Updated thread branching relationships')

    // Commit transaction
    await client.query('COMMIT')
    console.log('Sample data seeded successfully!')
  } catch (error) {
    // Rollback on error
    await client.query('ROLLBACK')
    console.error('Error seeding database:', error)
    throw error
  } finally {
    await client.end()
  }
}

// Run the seed function
seedDatabase().catch(console.error)
