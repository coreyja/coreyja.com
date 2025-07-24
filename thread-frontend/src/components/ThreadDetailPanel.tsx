import React, { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Thread, Stitch } from '../types'
import { threadsApi } from '../api/threads'
import { MessagesView } from './MessagesView'

interface ThreadDetailPanelProps {
  thread?: Thread
  stitch?: Stitch
  onClose: () => void
}

export const ThreadDetailPanel: React.FC<ThreadDetailPanelProps> = ({
  thread,
  stitch,
  onClose,
}) => {
  const [activeTab, setActiveTab] = useState<'details' | 'messages'>('details')

  const { data: messages, isLoading: messagesLoading } = useQuery({
    queryKey: ['threadMessages', thread?.thread_id],
    queryFn: () => threadsApi.getThreadMessages(thread!.thread_id),
    enabled: !!thread && activeTab === 'messages',
  })

  if (!thread && !stitch) return null

  return (
    <div
      style={{
        position: 'absolute',
        right: 0,
        top: 0,
        bottom: 0,
        width: '400px',
        background: 'white',
        borderLeft: '1px solid #ccc',
        padding: '20px',
        overflowY: 'auto',
        zIndex: 10,
      }}
    >
      <button
        onClick={onClose}
        style={{
          position: 'absolute',
          right: '10px',
          top: '10px',
          background: 'none',
          border: 'none',
          fontSize: '20px',
          cursor: 'pointer',
        }}
      >
        ×
      </button>

      {thread && (
        <div style={{ marginBottom: '20px' }}>
          <button
            onClick={() => setActiveTab('details')}
            style={{
              padding: '8px 16px',
              marginRight: '10px',
              border: '1px solid #ccc',
              borderRadius: '4px',
              background: activeTab === 'details' ? '#007bff' : 'white',
              color: activeTab === 'details' ? 'white' : 'black',
              cursor: 'pointer',
            }}
          >
            Details
          </button>
          <button
            onClick={() => setActiveTab('messages')}
            style={{
              padding: '8px 16px',
              border: '1px solid #ccc',
              borderRadius: '4px',
              background: activeTab === 'messages' ? '#007bff' : 'white',
              color: activeTab === 'messages' ? 'white' : 'black',
              cursor: 'pointer',
            }}
          >
            Messages
          </button>
        </div>
      )}

      {thread && activeTab === 'details' && (
        <div>
          <h2 style={{ marginTop: 0 }}>Thread Details</h2>
          <div style={{ marginBottom: '10px' }}>
            <strong>ID:</strong> {thread.thread_id}
          </div>
          <div style={{ marginBottom: '10px' }}>
            <strong>Goal:</strong> {thread.goal}
          </div>
          <div style={{ marginBottom: '10px' }}>
            <strong>Status:</strong> {thread.status}
          </div>
          <div style={{ marginBottom: '10px' }}>
            <strong>Type:</strong> {thread.thread_type}
          </div>
          {thread.tasks.length > 0 && (
            <div style={{ marginBottom: '10px' }}>
              <strong>Tasks:</strong>
              <ul style={{ marginTop: '5px' }}>
                {thread.tasks.map((task, index) => (
                  <li key={index} style={{ marginBottom: '5px' }}>
                    <span
                      style={{
                        color:
                          task.status === 'completed'
                            ? 'green'
                            : task.status === 'in_progress'
                              ? 'orange'
                              : 'gray',
                      }}
                    >
                      [{task.status}]
                    </span>{' '}
                    {task.description}
                  </li>
                ))}
              </ul>
            </div>
          )}
          {thread.result && (
            <div style={{ marginBottom: '10px' }}>
              <strong>Result:</strong>
              <pre
                style={{
                  background: '#f5f5f5',
                  padding: '10px',
                  borderRadius: '4px',
                  fontSize: '12px',
                  overflow: 'auto',
                }}
              >
                {JSON.stringify(thread.result, null, 2)}
              </pre>
            </div>
          )}
          {thread.discord_metadata && (
            <div style={{ marginBottom: '10px' }}>
              <h3>Discord Information</h3>
              <div style={{ marginBottom: '5px' }}>
                <strong>Discord Thread:</strong> {thread.discord_metadata.thread_name}
              </div>
              <div style={{ marginBottom: '5px' }}>
                <strong>Thread ID:</strong> {thread.discord_metadata.discord_thread_id}
              </div>
              <div style={{ marginBottom: '5px' }}>
                <strong>Channel ID:</strong> {thread.discord_metadata.channel_id}
              </div>
              <div style={{ marginBottom: '5px' }}>
                <strong>Guild ID:</strong> {thread.discord_metadata.guild_id}
              </div>
              <div style={{ marginBottom: '5px' }}>
                <strong>Created By:</strong> {thread.discord_metadata.created_by}
              </div>
              {thread.discord_metadata.participants.length > 0 && (
                <div style={{ marginBottom: '5px' }}>
                  <strong>Participants:</strong>
                  <ul style={{ marginTop: '5px', marginBottom: 0 }}>
                    {thread.discord_metadata.participants.map((p, i) => (
                      <li key={i}>{p}</li>
                    ))}
                  </ul>
                </div>
              )}
              {thread.discord_metadata.last_message_id && (
                <div style={{ marginBottom: '5px' }}>
                  <strong>Last Message ID:</strong> {thread.discord_metadata.last_message_id}
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {thread && activeTab === 'messages' && (
        <div>
          <h2 style={{ marginTop: 0 }}>Thread Messages</h2>
          {messagesLoading ? (
            <div>Loading messages...</div>
          ) : messages ? (
            <MessagesView messages={messages} />
          ) : (
            <div>No messages available</div>
          )}
        </div>
      )}

      {stitch && (
        <div>
          <h2 style={{ marginTop: 0 }}>Stitch Details</h2>
          <div style={{ marginBottom: '10px' }}>
            <strong>ID:</strong> {stitch.stitch_id}
          </div>
          <div style={{ marginBottom: '10px' }}>
            <strong>Type:</strong> {stitch.stitch_type}
          </div>
          {stitch.tool_name && (
            <div style={{ marginBottom: '10px' }}>
              <strong>Tool:</strong> {stitch.tool_name}
            </div>
          )}
          {stitch.llm_request && (
            <div style={{ marginBottom: '10px' }}>
              <strong>LLM Request:</strong>
              <pre
                style={{
                  background: '#f5f5f5',
                  padding: '10px',
                  borderRadius: '4px',
                  fontSize: '12px',
                  overflow: 'auto',
                }}
              >
                {JSON.stringify(stitch.llm_request, null, 2)}
              </pre>
            </div>
          )}
          {stitch.llm_response && (
            <div style={{ marginBottom: '10px' }}>
              <strong>LLM Response:</strong>
              <pre
                style={{
                  background: '#f5f5f5',
                  padding: '10px',
                  borderRadius: '4px',
                  fontSize: '12px',
                  overflow: 'auto',
                }}
              >
                {JSON.stringify(stitch.llm_response, null, 2)}
              </pre>
            </div>
          )}
          {stitch.tool_input && (
            <div style={{ marginBottom: '10px' }}>
              <strong>Tool Input:</strong>
              <pre
                style={{
                  background: '#f5f5f5',
                  padding: '10px',
                  borderRadius: '4px',
                  fontSize: '12px',
                  overflow: 'auto',
                }}
              >
                {JSON.stringify(stitch.tool_input, null, 2)}
              </pre>
            </div>
          )}
          {stitch.tool_output && (
            <div style={{ marginBottom: '10px' }}>
              <strong>Tool Output:</strong>
              <pre
                style={{
                  background: '#f5f5f5',
                  padding: '10px',
                  borderRadius: '4px',
                  fontSize: '12px',
                  overflow: 'auto',
                }}
              >
                {JSON.stringify(stitch.tool_output, null, 2)}
              </pre>
            </div>
          )}
        </div>
      )}
    </div>
  )
}
