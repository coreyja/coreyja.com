import React, { useState } from 'react'
import { ChevronDownIcon, ChevronRightIcon } from 'lucide-react'
import { Message, MessageContent } from '../types'

interface MessagesViewProps {
  messages: Message[]
}

interface CollapsibleJSONProps {
  data: unknown
  label?: string
  defaultExpanded?: boolean
}

const CollapsibleJSON: React.FC<CollapsibleJSONProps> = ({
  data,
  label,
  defaultExpanded = false,
}) => {
  const [isExpanded, setIsExpanded] = useState(defaultExpanded)

  return (
    <div className="border border-gray-300 rounded p-2 mb-2">
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="flex items-center w-full text-left font-mono text-sm hover:bg-gray-100 p-1 rounded"
      >
        {isExpanded ? (
          <ChevronDownIcon className="w-4 h-4 mr-1" />
        ) : (
          <ChevronRightIcon className="w-4 h-4 mr-1" />
        )}
        {label || 'JSON Data'}
      </button>
      {isExpanded && (
        <pre className="mt-2 p-2 bg-gray-50 rounded text-xs overflow-x-auto">
          {JSON.stringify(data, null, 2)}
        </pre>
      )}
    </div>
  )
}

const renderMessageContent = (content: MessageContent | MessageContent[]): React.ReactNode => {
  if (typeof content === 'string') {
    return <p className="whitespace-pre-wrap">{content}</p>
  }

  if (Array.isArray(content)) {
    return (
      <div className="space-y-2">
        {content.map((item, index) => (
          <div key={index}>{renderMessageContent(item)}</div>
        ))}
      </div>
    )
  }

  switch (content.type) {
    case 'text':
      return <p className="whitespace-pre-wrap">{content.text}</p>
    case 'tool_use':
      return (
        <div className="bg-blue-50 p-2 rounded">
          <div className="font-semibold text-blue-900">Tool Use: {content.name}</div>
          <div className="text-sm text-blue-700">ID: {content.id}</div>
          <CollapsibleJSON data={content.input} label="Input" />
        </div>
      )
    case 'tool_result':
      return (
        <div className="bg-green-50 p-2 rounded">
          <div className="font-semibold text-green-900">Tool Result</div>
          <div className="text-sm text-green-700">Tool Use ID: {content.tool_use_id}</div>
          <p className="mt-1 whitespace-pre-wrap">{content.content}</p>
        </div>
      )
    default:
      return <CollapsibleJSON data={content} label="Unknown Content Type" />
  }
}

export const MessagesView: React.FC<MessagesViewProps> = ({ messages }) => {
  return (
    <div className="space-y-4">
      {messages.map((message, index) => (
        <div
          key={index}
          className={`p-4 rounded-lg ${
            message.role === 'assistant'
              ? 'bg-gray-100 border-l-4 border-blue-500'
              : 'bg-white border-l-4 border-green-500'
          }`}
        >
          <div className="font-semibold mb-2 capitalize">{message.role}</div>
          <div className="text-sm">{renderMessageContent(message.content)}</div>
        </div>
      ))}
    </div>
  )
}
