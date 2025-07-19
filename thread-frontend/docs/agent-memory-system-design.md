# Agent Memory System Design

## Overview

This document outlines the design for a persistent memory system for our AI agent, inspired by Letta.ai (formerly MemGPT) and Void on Bluesky. The system will enable the agent to maintain context across conversations, build relationships with users, and continuously learn from interactions.

## Core Architecture

### 1. Memory Hierarchy

The memory system consists of three main layers:

#### 1.1 Core Memory Blocks (System Prompt)
- **Purpose**: Immediate working memory that's always accessible
- **Location**: Injected directly into the system prompt
- **Size**: Limited by context window constraints
- **Persistence**: Stored in database, loaded at conversation start

##### Memory Block Types:
- **Persona Block**: Agent's personality, behavioral guidelines, and self-concept
- **User Context Block**: Current user information and preferences
- **System State Block**: Current operational parameters and active goals
- **Custom Blocks**: Extensible for domain-specific needs

#### 1.2 Archival Memory (Long-term Storage)
- **Purpose**: Unbounded storage for historical information
- **Implementation**: Vector database with embeddings
- **Access**: Through dedicated memory tools
- **Features**:
  - Automatic chunking and embedding
  - Similarity search
  - Temporal indexing
  - Memory consolidation

#### 1.3 Relational Memory (People & Relationships)
- **Purpose**: Track individuals and their relationships
- **Structure**: Graph-based storage
- **Components**:
  - Person nodes with attributes
  - Relationship edges with metadata
  - Associated memories linked to both people and relationships

## Memory Block Structure

Each memory block contains:
```typescript
interface MemoryBlock {
  id: string;
  label: string;
  value: string;
  maxSize: number;
  isEditable: boolean;
  description?: string;
  lastModified: Date;
}
```

### Example Memory Blocks

```yaml
persona:
  label: "Agent Persona"
  value: |
    I am a helpful AI assistant with a focus on building meaningful
    relationships. I value authenticity, curiosity, and continuous
    learning. I adapt my communication style based on user preferences
    while maintaining my core values.
  maxSize: 500
  isEditable: true

user_context:
  label: "User Information"
  value: |
    Name: [To be learned]
    Preferences: [To be discovered]
    Communication Style: [To be observed]
    Current Projects: [To be tracked]
  maxSize: 1000
  isEditable: true
```

## Memory Management Tools

### 1. Core Memory Tools

#### `memory_read(block_name: string)`
Read the current value of a memory block.

#### `memory_edit(block_name: string, new_value: string)`
Update a memory block with new information.

#### `memory_append(block_name: string, additional_content: string)`
Add content to an existing memory block.

### 2. Archival Memory Tools

#### `archive_memory_add(content: string, metadata?: object)`
Store a new memory with automatic embedding and indexing.

#### `archive_memory_search(query: string, limit: number = 10)`
Search archival memory using semantic similarity.

#### `archive_memory_search_date(start_date: Date, end_date: Date)`
Retrieve memories from a specific time period.

### 3. Relational Memory Tools

#### `person_add(name: string, attributes: object)`
Create a new person entity.

#### `person_update(person_id: string, attributes: object)`
Update information about a person.

#### `person_attach_memory(person_id: string, memory_id: string)`
Link a memory to a specific person.

#### `relationship_create(person1_id: string, person2_id: string, type: string, metadata?: object)`
Create a relationship between two people.

#### `relationship_attach_memory(relationship_id: string, memory_id: string)`
Attach a memory to a relationship.

#### `relationship_query(filters: object)`
Query relationships based on various criteria.

## Memory Consolidation Process

### Automatic Consolidation
1. **Frequency**: Run after every N interactions or time period
2. **Process**:
   - Review recent memories
   - Identify patterns and recurring themes
   - Update core memory blocks with relevant insights
   - Archive detailed memories with proper categorization
   - Prune redundant or outdated information

### Manual Consolidation Tools

#### `memory_reflect(time_period: string)`
Agent reflects on memories from a specific period and generates insights.

#### `memory_reorganize(category: string)`
Reorganize memories within a specific category for better retrieval.

## Implementation Considerations

### 1. Database Schema

```sql
-- Core memory blocks
CREATE TABLE memory_blocks (
  id UUID PRIMARY KEY,
  label VARCHAR(255) NOT NULL,
  value TEXT NOT NULL,
  max_size INTEGER NOT NULL,
  is_editable BOOLEAN DEFAULT true,
  description TEXT,
  last_modified TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Archival memories
CREATE TABLE archival_memories (
  id UUID PRIMARY KEY,
  content TEXT NOT NULL,
  embedding VECTOR(1536),
  metadata JSONB,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- People
CREATE TABLE people (
  id UUID PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  attributes JSONB,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Relationships
CREATE TABLE relationships (
  id UUID PRIMARY KEY,
  person1_id UUID REFERENCES people(id),
  person2_id UUID REFERENCES people(id),
  relationship_type VARCHAR(100),
  metadata JSONB,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Memory associations
CREATE TABLE memory_associations (
  id UUID PRIMARY KEY,
  memory_id UUID REFERENCES archival_memories(id),
  entity_type VARCHAR(50), -- 'person' or 'relationship'
  entity_id UUID,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### 2. Privacy and Security
- Implement access controls for memory operations
- Encrypt sensitive memory content
- Provide user controls for memory deletion
- Audit trail for all memory modifications

### 3. Performance Optimization
- Cache frequently accessed memory blocks
- Implement efficient vector search algorithms
- Use database indexing for relationship queries
- Batch embedding operations

## User Experience Features

### 1. Memory Transparency
- Users can view what the agent remembers about them
- Ability to correct or delete specific memories
- Export personal data

### 2. Relationship Visualization
- Graph visualization of known people and relationships
- Timeline view of interactions
- Memory association viewer

### 3. Memory Health Indicators
- Memory usage statistics
- Consolidation status
- Memory quality metrics

## Future Enhancements

1. **Multi-Agent Memory Sharing**
   - Shared memory blocks between agents
   - Memory synchronization protocols
   - Privacy controls for shared memories

2. **Advanced Memory Reasoning**
   - Causal relationship detection
   - Temporal pattern recognition
   - Predictive memory retrieval

3. **Memory Templates**
   - Domain-specific memory structures
   - Industry-standard schemas
   - Customizable memory workflows

## Conclusion

This memory system design provides a robust foundation for creating AI agents with genuine long-term memory capabilities. By combining the structured approach of Letta's memory blocks with the relationship tracking inspired by Void, we can create agents that form meaningful connections with users while maintaining organized, accessible, and transparent memory systems.